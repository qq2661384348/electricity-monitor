//! 房间数据爬取器
//!
//! 负责从外部API获取房间数据，并实现1:N合并逻辑
//! 支持并发处理以提升性能

use anyhow::{Context, Result};
use futures::{stream, StreamExt, TryStreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;

use super::client::RoomClient;
use super::models::{
    ApartmentComponent, MergeStatistics, RoomData, RoomInfo, RoomListComponent, SchoolComponent,
    UpayResponse,
};
use super::parser;

/// 房间数据爬取器
///
/// 支持并发处理，通过Semaphore控制并发数
#[derive(Clone)]
pub struct RoomFetcher {
    /// HTTP客户端
    client: Arc<RoomClient>,
    /// 全局并发控制信号量
    semaphore: Arc<Semaphore>,
    /// 校区并发数
    campus_concurrency: usize,
    /// 建筑并发数
    building_concurrency: usize,
}

impl RoomFetcher {
    /// 创建新的爬取器实例（默认配置）
    pub fn new(client: Arc<RoomClient>) -> Self {
        Self::with_config(
            client, 20, // 默认最大并发数
            3,  // 默认校区并发数
            10, // 默认建筑并发数
        )
    }

    /// 创建带自定义配置的爬取器实例
    ///
    /// # 参数
    /// - `client`: HTTP客户端
    /// - `max_concurrent`: 最大并发请求数
    /// - `campus_concurrency`: 校区处理并发数
    /// - `building_concurrency`: 建筑处理并发数
    pub fn with_config(
        client: Arc<RoomClient>,
        max_concurrent: usize,
        campus_concurrency: usize,
        building_concurrency: usize,
    ) -> Self {
        Self {
            client,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            campus_concurrency,
            building_concurrency,
        }
    }

    /// 获取所有房间数据（已合并）
    ///
    /// 这是主要的公共接口，返回已经按roomid合并的数据
    ///
    /// # 返回
    /// Vec<RoomData> - 支持1:N映射的房间数据
    ///
    /// # 错误
    /// - HTTP请求失败
    /// - JSON解析失败
    /// - roomid转换失败（部分数据会跳过）
    pub async fn fetch_all(&self) -> Result<Vec<RoomData>> {
        tracing::info!("==== 开始获取房间数据 ====");

        // 1. Level 1: 获取校区列表
        let campuses = self
            .fetch_level1()
            .await
            .context("Level 1: 获取校区列表失败")?;

        tracing::info!("✓ Level 1 完成：获取到 {} 个校区", campuses.len());

        if campuses.is_empty() {
            tracing::warn!("未找到任何校区，返回空结果");
            return Ok(Vec::new());
        }

        // 2. Level 2-3: 并发处理校区（通过信号量控制并发数）
        let fetcher = self.clone();

        let all_rooms: Vec<RoomInfo> = stream::iter(campuses)
            .enumerate()
            .map(move |(idx, campus)| {
                let fetcher = fetcher.clone();
                async move {
                    tracing::info!(
                        "→ 并发处理校区 {}: {} (ID: {})",
                        idx + 1,
                        campus.school_name.as_deref().unwrap_or("未知校区"),
                        campus.school_id.as_deref().unwrap_or("")
                    );

                    match fetcher.fetch_campus_rooms(&campus).await {
                        Ok(rooms) => {
                            tracing::info!(
                                "  ✓ 校区 \"{}\" 完成：获取到 {} 个房间",
                                campus.school_name.as_deref().unwrap_or("未知校区"),
                                rooms.len()
                            );
                            Ok::<Vec<RoomInfo>, anyhow::Error>(rooms)
                        }
                        Err(e) => {
                            tracing::error!(
                                "  ✗ 校区 \"{}\" 失败: {:?}",
                                campus.school_name.as_deref().unwrap_or("未知校区"),
                                e
                            );
                            // 继续处理其他校区（优雅降级）
                            Ok::<Vec<RoomInfo>, anyhow::Error>(Vec::new())
                        }
                    }
                }
            })
            .buffer_unordered(self.campus_concurrency)
            .try_fold(Vec::new(), |mut acc, rooms| async {
                acc.extend(rooms);
                Ok(acc)
            })
            .await?;

        tracing::info!("获取到 {} 条原始记录", all_rooms.len());

        // 3. 按roomid合并（1:N场景处理）
        let (merged, stats) = self
            .group_by_roomid_from_info(all_rooms)
            .context("合并房间数据失败")?;

        // 4. 输出统计信息
        stats.log();

        // 5. 检查是否有数据
        if merged.is_empty() {
            tracing::warn!("未获取到任何有效房间数据");
        }

        tracing::info!(
            "==== 房间数据获取完成：共 {} 个有效roomid ====",
            merged.len()
        );

        Ok(merged)
    }

    /// Level 1: 获取校区列表
    async fn fetch_level1(&self) -> Result<Vec<SchoolComponent>> {
        tracing::debug!("Level 1: 请求校区列表");

        let json_str = self
            .client
            .fetch_room_tree()
            .await
            .context("Level 1 请求失败")?;

        let value = parser::safe_parse(&json_str).context("Level 1 JSON解析失败")?;

        let response: UpayResponse<SchoolComponent> =
            sonic_rs::from_value(&value).context("Level 1 UpayResponse转换失败")?;

        let campuses = response
            .into_data()
            .into_iter()
            .filter(|school| school.school_id.is_some() && school.school_name.is_some())
            .collect::<Vec<_>>();

        tracing::debug!("Level 1: 收到 {} 个校区", campuses.len());
        Ok(campuses)
    }

    /// Level 2: 获取单个校区的所有楼栋
    async fn fetch_buildings(&self, campus_id: &str) -> Result<Vec<ApartmentComponent>> {
        tracing::debug!("Level 2: 请求楼栋列表（校区 ID: {}）", campus_id);

        let json_str = self
            .client
            .fetch_apartments(campus_id)
            .await
            .context("Level 2 请求失败")?;

        let value = parser::safe_parse(&json_str).context("Level 2 JSON解析失败")?;

        let response: UpayResponse<ApartmentComponent> =
            sonic_rs::from_value(&value).context("Level 2 UpayResponse转换失败")?;

        let buildings = response
            .into_data()
            .into_iter()
            .filter(|apart| apart.apart_id.is_some() && apart.apart_name.is_some())
            .collect::<Vec<_>>();

        tracing::debug!("Level 2: 收到 {} 个建筑", buildings.len());
        Ok(buildings)
    }

    /// Level 3: 获取房间列表
    async fn fetch_rooms(&self, apart_id: &str, room_path: &str) -> Result<Vec<RoomInfo>> {
        tracing::debug!("Level 3: 请求房间列表（楼栋 ID: {}）", apart_id);

        let json_str = self
            .client
            .fetch_rooms(apart_id)
            .await
            .context("Level 3 请求失败")?;

        let value = parser::safe_parse(&json_str).context("Level 3 JSON解析失败")?;

        let response: UpayResponse<RoomListComponent> =
            sonic_rs::from_value(&value).context("Level 3 UpayResponse转换失败")?;

        let room_components = response.into_data();

        tracing::debug!("Level 3: 收到 {} 个房间", room_components.len());

        // 构建最终的房间信息
        let rooms: Vec<RoomInfo> = room_components
            .into_iter()
            .filter_map(|component| {
                let room_name = component.room_name?;
                let room_id = component.room_id?;
                let full_path = format!("{}/{}", room_path, room_name).replace("//", "/");

                Some(RoomInfo {
                    roompath: full_path,
                    roomid: room_id,
                })
            })
            .collect();

        Ok(rooms)
    }

    /// 处理单个校区（获取其下所有房间）
    ///
    /// 使用并发处理建筑，提升性能
    async fn fetch_campus_rooms(&self, campus: &SchoolComponent) -> Result<Vec<RoomInfo>> {
        let campus_id = campus.school_id.as_deref().context("校区 SchoolId 为空")?;
        let campus_name = campus
            .school_name
            .as_deref()
            .context("校区 SchoolName 为空")?;

        // Level 2: 获取建筑列表
        let buildings = self.fetch_buildings(campus_id).await?;

        tracing::debug!("  校区 \"{}\" 有 {} 个建筑", campus_name, buildings.len());

        if buildings.is_empty() {
            return Ok(Vec::new());
        }

        // 并发处理建筑
        let campus_name = campus_name.to_string();
        let fetcher = self.clone();

        let rooms: Vec<RoomInfo> = stream::iter(buildings)
            .map(|building| {
                let fetcher = fetcher.clone();
                let campus_name = campus_name.clone();
                async move {
                    // 获取信号量许可
                    let _permit = fetcher
                        .semaphore
                        .acquire()
                        .await
                        .map_err(|e| anyhow::anyhow!("获取信号量失败: {}", e))?;

                    tracing::debug!(
                        "    → 并发处理建筑: {} (ID: {})",
                        building.apart_name.as_deref().unwrap_or("未知楼栋"),
                        building.apart_id.as_deref().unwrap_or("")
                    );

                    match fetcher.fetch_building_rooms(&campus_name, &building).await {
                        Ok(building_rooms) => {
                            tracing::debug!(
                                "    ✓ 建筑 \"{}\" 完成：{} 个房间",
                                building.apart_name.as_deref().unwrap_or("未知楼栋"),
                                building_rooms.len()
                            );
                            Ok(building_rooms)
                        }
                        Err(e) => {
                            tracing::warn!("    ✗ 建筑处理失败: {:?}", e);
                            // 返回空列表，继续处理其他建筑
                            Ok::<Vec<RoomInfo>, anyhow::Error>(Vec::new())
                        }
                    }
                }
            })
            .buffer_unordered(self.building_concurrency)
            .try_fold(Vec::new(), |mut acc, rooms| async {
                acc.extend(rooms);
                Ok(acc)
            })
            .await?;

        tracing::debug!(
            "  ✓ 校区 \"{}\" 完成：获取到 {} 个房间",
            campus_name,
            rooms.len()
        );

        Ok(rooms)
    }

    /// 获取单个建筑的所有房间（Level 3 + 4）
    async fn fetch_building_rooms(
        &self,
        campus_name: &str,
        building: &ApartmentComponent,
    ) -> Result<Vec<RoomInfo>> {
        let apart_id = building.apart_id.as_deref().context("楼栋 ApartID 为空")?;
        let apart_name = building
            .apart_name
            .as_deref()
            .context("楼栋 ApartName 为空")?;
        let room_path = format!("{}/{}", campus_name, apart_name);
        let all_rooms = self.fetch_rooms(apart_id, &room_path).await?;

        tracing::debug!(
            "      ✓ 建筑 \"{}\" 完成：获取到 {} 个房间",
            apart_name,
            all_rooms.len()
        );

        Ok(all_rooms)
    }

    /// 按roomid合并数据（核心算法）⭐
    ///
    /// 将扁平的RawRoomInfo列表按roomid分组，处理1:N映射场景
    ///
    /// # 算法步骤
    /// 1. 使用HashMap<i64, Vec<String>>按roomid分组
    /// 2. 处理roomid类型转换（String → i64）
    /// 3. 统计转换失败的记录
    /// 4. 生成MergeStatistics
    ///
    /// # 参数
    /// - `raw`: 原始扁平数据列表
    ///
    /// # 返回
    /// (Vec<RoomData>, MergeStatistics) - 合并后的数据和统计信息
    ///
    /// # 示例
    /// ```ignore
    /// let raw = vec![
    ///     RawRoomInfo { roompath: "path1".into(), roomid: "101".into() },
    ///     RawRoomInfo { roompath: "path2".into(), roomid: "101".into() },  // 同一roomid
    ///     RawRoomInfo { roompath: "path3".into(), roomid: "102".into() },
    /// ];
    ///
    /// let (merged, stats) = fetcher.group_by_roomid(raw)?;
    ///
    /// assert_eq!(merged.len(), 2);  // 2个唯一roomid
    /// assert_eq!(merged[0].path_count, 2);  // roomid=101有2个路径
    /// assert_eq!(stats.multi_path_count, 1);  // 1个1:N场景
    /// ```
    fn group_by_roomid_from_info(
        &self,
        raw: Vec<RoomInfo>,
    ) -> Result<(Vec<RoomData>, MergeStatistics)> {
        let raw_count = raw.len();
        let mut map: HashMap<i64, Vec<String>> = HashMap::new();
        let mut parse_error_count = 0;

        // 遍历原始数据，按roomid分组
        for room in raw {
            match room.roomid.parse::<i64>() {
                Ok(roomid) => {
                    // roomid转换成功，添加到对应的路径列表
                    map.entry(roomid).or_default().push(room.roompath);
                }
                Err(e) => {
                    // roomid转换失败，记录警告并跳过
                    tracing::warn!(
                        "roomid转换失败: roomid='{}', roompath='{}', error={}",
                        room.roomid,
                        room.roompath,
                        e
                    );
                    parse_error_count += 1;
                }
            }
        }

        // 转换HashMap为Vec<RoomData>
        let mut merged: Vec<RoomData> = map
            .into_iter()
            .map(|(roomid, roompaths)| RoomData::new(roomid, roompaths))
            .collect();

        // 按roomid排序（便于测试和调试）
        merged.sort_by_key(|r| r.roomid);

        // 计算统计信息
        let stats = MergeStatistics::calculate(raw_count, &merged, parse_error_count);

        Ok((merged, stats))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CrawlerConfig;
    use crate::infrastructure::electricity::fetcher::RoomBatchFetcher;

    #[test]
    fn test_group_by_roomid_normal() {
        let client = Arc::new(RoomClient::new(&CrawlerConfig::default()).unwrap());
        let fetcher = RoomFetcher::new(client);

        let raw = vec![
            RoomInfo {
                roompath: "path1".into(),
                roomid: "101".into(),
            },
            RoomInfo {
                roompath: "path2".into(),
                roomid: "102".into(),
            },
            RoomInfo {
                roompath: "path3".into(),
                roomid: "103".into(),
            },
        ];

        let (merged, stats) = fetcher.group_by_roomid_from_info(raw).unwrap();

        assert_eq!(merged.len(), 3);
        assert_eq!(stats.raw_count, 3);
        assert_eq!(stats.unique_roomid_count, 3);
        assert_eq!(stats.multi_path_count, 0); // 没有1:N场景
        assert_eq!(stats.parse_error_count, 0);
    }

    #[test]
    fn test_group_by_roomid_with_duplicates() {
        let client = Arc::new(RoomClient::new(&CrawlerConfig::default()).unwrap());
        let fetcher = RoomFetcher::new(client);

        let raw = vec![
            RoomInfo {
                roompath: "path1".into(),
                roomid: "101".into(),
            },
            RoomInfo {
                roompath: "path2".into(),
                roomid: "101".into(),
            }, // 同一roomid
            RoomInfo {
                roompath: "path3".into(),
                roomid: "102".into(),
            },
        ];

        let (merged, stats) = fetcher.group_by_roomid_from_info(raw).unwrap();

        assert_eq!(merged.len(), 2); // 2个唯一roomid
        assert_eq!(stats.multi_path_count, 1); // 1个1:N场景

        // 验证roomid=101有2个路径
        let room_101 = merged.iter().find(|r| r.roomid == 101).unwrap();
        assert_eq!(room_101.path_count, 2);
    }

    #[test]
    fn test_group_by_roomid_with_parse_errors() {
        let client = Arc::new(RoomClient::new(&CrawlerConfig::default()).unwrap());
        let fetcher = RoomFetcher::new(client);

        let raw = vec![
            RoomInfo {
                roompath: "path1".into(),
                roomid: "101".into(),
            },
            RoomInfo {
                roompath: "path2".into(),
                roomid: "invalid".into(),
            }, // 无效roomid
            RoomInfo {
                roompath: "path3".into(),
                roomid: "abc123".into(),
            }, // 无效roomid
        ];

        let (merged, stats) = fetcher.group_by_roomid_from_info(raw).unwrap();

        assert_eq!(merged.len(), 1); // 只有1个有效roomid
        assert_eq!(stats.parse_error_count, 2); // 2个转换失败
    }

    #[test]
    fn test_group_by_roomid_accepts_large_upay_room_id() {
        let client = Arc::new(RoomClient::new(&CrawlerConfig::default()).unwrap());
        let fetcher = RoomFetcher::new(client);

        let raw = vec![RoomInfo {
            roompath: "文昌校区/北区4栋公寓/107".into(),
            roomid: "982318536531644416".into(),
        }];

        let (merged, stats) = fetcher.group_by_roomid_from_info(raw).unwrap();

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].roomid.to_string(), "982318536531644416");
        assert_eq!(stats.parse_error_count, 0);
    }

    #[test]
    fn test_group_by_roomid_deduplication() {
        let client = Arc::new(RoomClient::new(&CrawlerConfig::default()).unwrap());
        let fetcher = RoomFetcher::new(client);

        let raw = vec![
            RoomInfo {
                roompath: "path1".into(),
                roomid: "101".into(),
            },
            RoomInfo {
                roompath: "path1".into(),
                roomid: "101".into(),
            }, // 完全重复
            RoomInfo {
                roompath: "path2".into(),
                roomid: "101".into(),
            },
        ];

        let (merged, _stats) = fetcher.group_by_roomid_from_info(raw).unwrap();

        assert_eq!(merged.len(), 1);

        let room_101 = &merged[0];
        assert_eq!(room_101.path_count, 2); // 去重后只有2个路径
    }

    #[tokio::test]
    async fn test_real_upay_room_tree_room_id_and_electricity_e2e() {
        if std::env::var("RUN_EXTERNAL_INTEGRATION_TESTS").is_err() {
            println!("跳过真实 Upay e2e：设置 RUN_EXTERNAL_INTEGRATION_TESTS=1 以启用");
            return;
        }

        let config = CrawlerConfig {
            timeout_seconds: 20,
            connect_timeout_seconds: 10,
            max_retries: 2,
            concurrency: 8,
            ..CrawlerConfig::default()
        };
        let client = Arc::new(RoomClient::new(&config).expect("创建真实 RoomClient 失败"));
        let fetcher = RoomFetcher::with_config(client, 8, 2, 4);

        let rooms = fetcher
            .fetch_all()
            .await
            .expect("真实房间树 e2e 应能获取房间");
        assert!(!rooms.is_empty(), "真实房间树 e2e 应至少返回一个房间");

        let room = rooms
            .iter()
            .find(|room| room.roomid > i32::MAX as i64)
            .or_else(|| rooms.first())
            .expect("真实房间树 e2e 应至少返回一个可用 roomid");
        assert!(
            !room.primary_roompath.trim().is_empty(),
            "真实房间树 e2e 返回的主路径不应为空"
        );

        let electricity_fetcher = RoomBatchFetcher::new(
            "https://upayadmin.gyruibo.cn/UpayManage/Home/GetRoom?roomid=".to_string(),
            1,
        )
        .expect("创建真实电费获取器失败");
        let electricity = electricity_fetcher.fetch_batch(vec![room.roomid]).await;

        assert!(
            electricity.contains_key(&room.roomid),
            "真实电费 e2e 应能通过房间树得到的 roomid={} 获取电费；roompath={}",
            room.roomid,
            room.primary_roompath
        );

        println!(
            "真实 Upay e2e 通过：rooms={}, roomid={}, roompath={}, electricity={}",
            rooms.len(),
            room.roomid,
            room.primary_roompath,
            electricity[&room.roomid]
        );
    }
}
