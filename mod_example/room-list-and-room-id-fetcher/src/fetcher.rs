//! 核心爬取逻辑
//!
//! 实现4层级联爬取：校区 → 建筑 → 楼层 → 房间

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::client::RoomClient;
use crate::models::{RoomComponent, RoomInfo};

/// 房间信息爬取器
///
/// 管理整个爬取流程，包括：
/// - 多层级数据获取
/// - 并发控制（通过 Semaphore）
/// - 错误处理和恢复
pub struct RoomFetcher {
    /// HTTP 客户端（Arc 包装用于在异步任务间共享）
    client: Arc<RoomClient>,

    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
}

impl RoomFetcher {
    /// 创建新的爬取器实例
    ///
    /// # 参数
    /// - `client`: HTTP 客户端
    /// - `max_concurrent`: 最大并发数（建议 50）
    pub fn new(client: RoomClient, max_concurrent: usize) -> Self {
        tracing::info!("初始化爬取器，最大并发数: {}", max_concurrent);

        Self {
            client: Arc::new(client),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    /// 获取所有房间信息（4层级联入口）
    ///
    /// 执行流程：
    /// 1. Level 1: 获取校区列表（串行）
    /// 2. Level 2-4: 并发获取建筑、楼层、房间
    ///
    /// # 返回
    /// - `Ok(Vec<RoomInfo>)`: 所有房间信息
    /// - `Err`: 爬取过程中的错误
    pub async fn fetch_all(&self) -> Result<Vec<RoomInfo>> {
        tracing::info!("=== 开始获取房间信息 ===");

        // Level 1: 获取校区列表（串行）
        let campuses = self.fetch_level1().await.context("获取校区列表失败")?;

        tracing::info!("✓ Level 1 完成：获取到 {} 个校区", campuses.len());

        if campuses.is_empty() {
            tracing::warn!("未找到任何校区，返回空结果");
            return Ok(Vec::new());
        }

        // Level 2-4: 并发处理每个校区
        let mut all_rooms = Vec::new();

        for (idx, campus) in campuses.iter().enumerate() {
            tracing::info!(
                "→ 处理校区 {}/{}: {} (ID: {})",
                idx + 1,
                campuses.len(),
                campus.dep_name,
                campus.room_dep_id
            );

            match self.fetch_campus(campus).await {
                Ok(rooms) => {
                    tracing::info!(
                        "  ✓ 校区 \"{}\" 完成：获取到 {} 个房间",
                        campus.dep_name,
                        rooms.len()
                    );
                    all_rooms.extend(rooms);
                }
                Err(e) => {
                    tracing::error!("  ✗ 校区 \"{}\" 失败: {:?}", campus.dep_name, e);
                    // 继续处理其他校区（优雅降级）
                }
            }
        }

        tracing::info!("=== 爬取完成：共获取到 {} 个房间 ===", all_rooms.len());
        Ok(all_rooms)
    }

    /// Level 1: 获取校区列表
    async fn fetch_level1(&self) -> Result<Vec<RoomComponent>> {
        tracing::debug!("Level 1: 请求校区列表");

        let response = self
            .client
            .fetch_tree("yzm=123&Id=000&level=1")
            .await
            .context("Level 1 请求失败")?;

        let campuses = response.component.unwrap_or_default();

        tracing::debug!("Level 1: 收到 {} 个校区", campuses.len());
        Ok(campuses)
    }

    /// Level 2: 获取单个校区的所有建筑
    async fn fetch_buildings(&self, campus_id: &str) -> Result<Vec<RoomComponent>> {
        tracing::debug!("Level 2: 请求建筑列表（校区 ID: {}）", campus_id);

        let params = format!("yzm=123&RoomDepId={}&level=2&floor=0", campus_id);
        let response = self
            .client
            .fetch_tree(&params)
            .await
            .context("Level 2 请求失败")?;

        let buildings = response.component.unwrap_or_default();

        tracing::debug!("Level 2: 收到 {} 个建筑", buildings.len());
        Ok(buildings)
    }

    /// 处理单个校区（获取其下所有房间）
    ///
    /// 流程：
    /// 1. 获取该校区的所有建筑（Level 2）
    /// 2. 并发处理每个建筑（Level 3-4）
    async fn fetch_campus(&self, campus: &RoomComponent) -> Result<Vec<RoomInfo>> {
        // Level 2: 获取建筑列表
        let buildings = self.fetch_buildings(&campus.room_dep_id).await?;

        tracing::debug!(
            "  校区 \"{}\" 有 {} 个建筑",
            campus.dep_name,
            buildings.len()
        );

        if buildings.is_empty() {
            return Ok(Vec::new());
        }

        // Level 3-4: 并发处理每个建筑
        let mut tasks = Vec::new();

        for building in buildings {
            let client = self.client.clone();
            let semaphore = self.semaphore.clone();
            let campus_name = campus.dep_name.clone();

            // 生成异步任务
            let task = tokio::spawn(async move {
                // 获取信号量许可（限流）
                let _permit = semaphore.acquire().await.unwrap();

                tracing::debug!(
                    "    → 处理建筑: {} (ID: {})",
                    building.dep_name,
                    building.room_dep_id
                );

                // 获取该建筑的所有房间（Level 3-4）
                fetch_building_rooms(client, &campus_name, &building).await
            });

            tasks.push(task);
        }

        // 等待所有任务完成并收集结果
        let mut rooms = Vec::new();

        for task in tasks {
            match task.await {
                Ok(Ok(building_rooms)) => {
                    rooms.extend(building_rooms);
                }
                Ok(Err(e)) => {
                    tracing::warn!("    ✗ 建筑处理失败: {:?}", e);
                    // 继续处理其他建筑
                }
                Err(e) => {
                    tracing::error!("    ✗ 任务执行失败: {:?}", e);
                }
            }
        }

        Ok(rooms)
    }
}

/// 获取单个建筑的所有房间（Level 3 + 4）
///
/// # 参数
/// - `client`: HTTP 客户端（Arc 包装）
/// - `campus_name`: 校区名称
/// - `building`: 建筑信息
///
/// # 返回
/// 该建筑的所有房间信息
async fn fetch_building_rooms(
    client: Arc<RoomClient>,
    campus_name: &str,
    building: &RoomComponent,
) -> Result<Vec<RoomInfo>> {
    // Level 3: 获取楼层列表
    let floors = fetch_floors(&client, &building.room_dep_id).await?;

    tracing::debug!(
        "      建筑 \"{}\" 有 {} 个楼层",
        building.dep_name,
        floors.len()
    );

    if floors.is_empty() {
        return Ok(Vec::new());
    }

    // Level 4: 并发获取所有楼层的房间
    let mut tasks = Vec::new();

    for floor in floors {
        let client_clone = Arc::clone(&client);
        let room_path = format!("{}/{}/{}", campus_name, building.dep_name, floor.dep_name);

        // 生成异步任务
        let task =
            tokio::spawn(
                async move { fetch_rooms(client_clone, &floor.room_dep_id, &room_path).await },
            );

        tasks.push(task);
    }

    // 等待所有任务完成并收集结果
    let mut all_rooms = Vec::new();

    for task in tasks {
        match task.await {
            Ok(Ok(rooms)) => {
                all_rooms.extend(rooms);
            }
            Ok(Err(e)) => {
                tracing::warn!("      ✗ 楼层处理失败: {:?}", e);
                // 继续处理其他楼层
            }
            Err(e) => {
                tracing::error!("      ✗ 任务执行失败: {:?}", e);
            }
        }
    }

    tracing::debug!(
        "      ✓ 建筑 \"{}\" 完成：获取到 {} 个房间",
        building.dep_name,
        all_rooms.len()
    );

    Ok(all_rooms)
}

/// Level 3: 获取楼层列表
async fn fetch_floors(client: &RoomClient, building_id: &str) -> Result<Vec<RoomComponent>> {
    tracing::debug!("      Level 3: 请求楼层列表（建筑 ID: {}）", building_id);

    let params = format!("yzm=123&RoomDepId={}&level=3&floor=0", building_id);
    let response = client
        .fetch_tree(&params)
        .await
        .context("Level 3 请求失败")?;

    let floors = response.component.unwrap_or_default();

    tracing::debug!("      Level 3: 收到 {} 个楼层", floors.len());
    Ok(floors)
}

/// Level 4: 获取房间列表
async fn fetch_rooms(
    client: Arc<RoomClient>,
    floor_id: &str,
    room_path: &str,
) -> Result<Vec<RoomInfo>> {
    tracing::debug!("        Level 4: 请求房间列表（楼层 ID: {}）", floor_id);

    let params = format!("yzm=123&RoomDepId={}&level=4", floor_id);
    let response = client
        .fetch_tree(&params)
        .await
        .context("Level 4 请求失败")?;

    let room_components = response.component.unwrap_or_default();

    tracing::debug!("        Level 4: 收到 {} 个房间", room_components.len());

    // 构建最终的房间信息
    let rooms: Vec<RoomInfo> = room_components
        .into_iter()
        .map(|component| {
            // 构建完整路径：校区/建筑/楼层/房间
            let full_path = format!("{}/{}", room_path, component.dep_name).replace("//", "/"); // 清理可能的双斜杠

            RoomInfo {
                roompath: full_path,
                roomid: component.room_dep_id,
            }
        })
        .collect();

    Ok(rooms)
}
