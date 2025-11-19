//! 房间数据爬取器
//!
//! 负责从外部API获取房间数据，并实现1:N合并逻辑

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;

use super::client::RoomClient;
use super::models::{ApiResponse, MergeStatistics, RoomComponent, RoomData, RoomInfo};
use super::parser;

/// 房间数据爬取器
pub struct RoomFetcher {
    client: Arc<RoomClient>,
}

impl RoomFetcher {
    /// 创建新的爬取器实例
    pub fn new(client: Arc<RoomClient>) -> Self {
        Self { client }
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
        let campuses = self.fetch_level1().await
            .context("Level 1: 获取校区列表失败")?;
        
        tracing::info!("✓ Level 1 完成：获取到 {} 个校区", campuses.len());
        
        if campuses.is_empty() {
            tracing::warn!("未找到任何校区，返回空结果");
            return Ok(Vec::new());
        }
        
        // 2. Level 2-4: 顺序处理每个校区（避免服务器过载）
        let mut all_rooms = Vec::new();
        
        for (idx, campus) in campuses.iter().enumerate() {
            tracing::info!(
                "→ 处理校区 {}/{}: {} (ID: {})",
                idx + 1,
                campuses.len(),
                campus.dep_name,
                campus.room_dep_id
            );
            
            match self.fetch_campus_rooms(campus).await {
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
            
            // ⭐ 避免过快请求，每个校区间延迟200ms
            if idx + 1 < campuses.len() {
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
        }
        
        tracing::info!("获取到 {} 条原始记录", all_rooms.len());
        
        // 3. 按roomid合并（1:N场景处理）
        let (merged, stats) = self.group_by_roomid_from_info(all_rooms)
            .context("合并房间数据失败")?;
        
        // 4. 输出统计信息
        stats.log();
        
        // 5. 检查是否有数据
        if merged.is_empty() {
            tracing::warn!("未获取到任何有效房间数据");
        }
        
        tracing::info!("==== 房间数据获取完成：共 {} 个有效roomid ====", merged.len());
        
        Ok(merged)
    }
    
    /// Level 1: 获取校区列表
    async fn fetch_level1(&self) -> Result<Vec<RoomComponent>> {
        tracing::debug!("Level 1: 请求校区列表");
        
        let json_str = self.client.fetch_room_tree()
            .await
            .context("Level 1 请求失败")?;
        
        let value = parser::safe_parse(&json_str)
            .context("Level 1 JSON解析失败")?;
        
        let response: ApiResponse = sonic_rs::from_value(&value)
            .context("Level 1 ApiResponse转换失败")?;
        
        let campuses = response.component.unwrap_or_default();
        
        tracing::debug!("Level 1: 收到 {} 个校区", campuses.len());
        Ok(campuses)
    }
    
    /// Level 2: 获取单个校区的所有建筑
    async fn fetch_buildings(&self, campus_id: &str) -> Result<Vec<RoomComponent>> {
        tracing::debug!("Level 2: 请求建筑列表（校区 ID: {}）", campus_id);
        
        let params = format!("yzm=123&RoomDepId={}&level=2&floor=0", campus_id);
        let json_str = self.client.fetch_tree(&params)
            .await
            .context("Level 2 请求失败")?;
        
        let value = parser::safe_parse(&json_str)
            .context("Level 2 JSON解析失败")?;
        
        let response: ApiResponse = sonic_rs::from_value(&value)
            .context("Level 2 ApiResponse转换失败")?;
        
        let buildings = response.component.unwrap_or_default();
        
        tracing::debug!("Level 2: 收到 {} 个建筑", buildings.len());
        Ok(buildings)
    }
    
    /// Level 3: 获取楼层列表
    async fn fetch_floors(&self, building_id: &str) -> Result<Vec<RoomComponent>> {
        tracing::debug!("Level 3: 请求楼层列表（建筑 ID: {}）", building_id);
        
        let params = format!("yzm=123&RoomDepId={}&level=3&floor=0", building_id);
        let json_str = self.client.fetch_tree(&params)
            .await
            .context("Level 3 请求失败")?;
        
        let value = parser::safe_parse(&json_str)
            .context("Level 3 JSON解析失败")?;
        
        let response: ApiResponse = sonic_rs::from_value(&value)
            .context("Level 3 ApiResponse转换失败")?;
        
        let floors = response.component.unwrap_or_default();
        
        tracing::debug!("Level 3: 收到 {} 个楼层", floors.len());
        Ok(floors)
    }
    
    /// Level 4: 获取房间列表
    async fn fetch_rooms(
        &self,
        floor_id: &str,
        room_path: &str,
    ) -> Result<Vec<RoomInfo>> {
        tracing::debug!("Level 4: 请求房间列表（楼层 ID: {}）", floor_id);
        
        let params = format!("yzm=123&RoomDepId={}&level=4", floor_id);
        let json_str = self.client.fetch_tree(&params)
            .await
            .context("Level 4 请求失败")?;
        
        let value = parser::safe_parse(&json_str)
            .context("Level 4 JSON解析失败")?;
        
        let response: ApiResponse = sonic_rs::from_value(&value)
            .context("Level 4 ApiResponse转换失败")?;
        
        let room_components = response.component.unwrap_or_default();
        
        tracing::debug!("Level 4: 收到 {} 个房间", room_components.len());
        
        // 构建最终的房间信息
        let rooms: Vec<RoomInfo> = room_components
            .into_iter()
            .map(|component| {
                // 构建完整路径：校区/建筑/楼层/房间
                let full_path = format!("{}/{}", room_path, component.dep_name)
                    .replace("//", "/"); // 清理可能的双斜杠
                
                RoomInfo {
                    roompath: full_path,
                    roomid: component.room_dep_id,
                }
            })
            .collect();
        
        Ok(rooms)
    }
    
    /// 处理单个校区（获取其下所有房间）
    async fn fetch_campus_rooms(&self, campus: &RoomComponent) -> Result<Vec<RoomInfo>> {
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
        
        // Level 3-4: 顺序处理每个建筑（避免过多并发）
        let mut rooms = Vec::new();
        
        for (idx, building) in buildings.iter().enumerate() {
            tracing::debug!(
                "    → 处理建筑 {}/{}: {} (ID: {})",
                idx + 1,
                buildings.len(),
                building.dep_name,
                building.room_dep_id
            );
            
            match self.fetch_building_rooms(&campus.dep_name, building).await {
                Ok(building_rooms) => {
                    rooms.extend(building_rooms);
                }
                Err(e) => {
                    tracing::warn!("    ✗ 建筑处理失败: {:?}", e);
                    // 继续处理其他建筑
                }
            }
            
            // ⭐ 避免过快请求，每个建筑间延迟100ms
            if idx + 1 < buildings.len() {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
        
        Ok(rooms)
    }
    
    /// 获取单个建筑的所有房间（Level 3 + 4）
    async fn fetch_building_rooms(
        &self,
        campus_name: &str,
        building: &RoomComponent,
    ) -> Result<Vec<RoomInfo>> {
        // Level 3: 获取楼层列表
        let floors = self.fetch_floors(&building.room_dep_id).await?;
        
        tracing::debug!(
            "      建筑 \"{}\" 有 {} 个楼层",
            building.dep_name,
            floors.len()
        );
        
        if floors.is_empty() {
            return Ok(Vec::new());
        }
        
        // Level 4: 顺序获取所有楼层的房间
        let mut all_rooms = Vec::new();
        
        for floor in floors {
            let room_path = format!("{}/{}/{}", campus_name, building.dep_name, floor.dep_name);
            
            match self.fetch_rooms(&floor.room_dep_id, &room_path).await {
                Ok(rooms) => {
                    all_rooms.extend(rooms);
                }
                Err(e) => {
                    tracing::warn!("      ✗ 楼层处理失败: {:?}", e);
                    // 继续处理其他楼层
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
    
    /// 按roomid合并数据（核心算法）⭐
    /// 
    /// 将扁平的RawRoomInfo列表按roomid分组，处理1:N映射场景
    /// 
    /// # 算法步骤
    /// 1. 使用HashMap<i32, Vec<String>>按roomid分组
    /// 2. 处理roomid类型转换（String → i32）
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
    fn group_by_roomid_from_info(&self, raw: Vec<RoomInfo>) -> Result<(Vec<RoomData>, MergeStatistics)> {
        let raw_count = raw.len();
        let mut map: HashMap<i32, Vec<String>> = HashMap::new();
        let mut parse_error_count = 0;
        
        // 遍历原始数据，按roomid分组
        for room in raw {
            match room.roomid.parse::<i32>() {
                Ok(roomid) => {
                    // roomid转换成功，添加到对应的路径列表
                    map.entry(roomid)
                        .or_default()
                        .push(room.roompath);
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

    #[test]
    fn test_group_by_roomid_normal() {
        let client = Arc::new(RoomClient::new(&CrawlerConfig::default()).unwrap());
        let fetcher = RoomFetcher::new(client);
        
        let raw = vec![
            RoomInfo { roompath: "path1".into(), roomid: "101".into() },
            RoomInfo { roompath: "path2".into(), roomid: "102".into() },
            RoomInfo { roompath: "path3".into(), roomid: "103".into() },
        ];
        
        let (merged, stats) = fetcher.group_by_roomid_from_info(raw).unwrap();
        
        assert_eq!(merged.len(), 3);
        assert_eq!(stats.raw_count, 3);
        assert_eq!(stats.unique_roomid_count, 3);
        assert_eq!(stats.multi_path_count, 0);  // 没有1:N场景
        assert_eq!(stats.parse_error_count, 0);
    }

    #[test]
    fn test_group_by_roomid_with_duplicates() {
        let client = Arc::new(RoomClient::new(&CrawlerConfig::default()).unwrap());
        let fetcher = RoomFetcher::new(client);
        
        let raw = vec![
            RoomInfo { roompath: "path1".into(), roomid: "101".into() },
            RoomInfo { roompath: "path2".into(), roomid: "101".into() },  // 同一roomid
            RoomInfo { roompath: "path3".into(), roomid: "102".into() },
        ];
        
        let (merged, stats) = fetcher.group_by_roomid_from_info(raw).unwrap();
        
        assert_eq!(merged.len(), 2);  // 2个唯一roomid
        assert_eq!(stats.multi_path_count, 1);  // 1个1:N场景
        
        // 验证roomid=101有2个路径
        let room_101 = merged.iter().find(|r| r.roomid == 101).unwrap();
        assert_eq!(room_101.path_count, 2);
    }

    #[test]
    fn test_group_by_roomid_with_parse_errors() {
        let client = Arc::new(RoomClient::new(&CrawlerConfig::default()).unwrap());
        let fetcher = RoomFetcher::new(client);
        
        let raw = vec![
            RoomInfo { roompath: "path1".into(), roomid: "101".into() },
            RoomInfo { roompath: "path2".into(), roomid: "invalid".into() },  // 无效roomid
            RoomInfo { roompath: "path3".into(), roomid: "abc123".into() },  // 无效roomid
        ];
        
        let (merged, stats) = fetcher.group_by_roomid_from_info(raw).unwrap();
        
        assert_eq!(merged.len(), 1);  // 只有1个有效roomid
        assert_eq!(stats.parse_error_count, 2);  // 2个转换失败
    }

    #[test]
    fn test_group_by_roomid_deduplication() {
        let client = Arc::new(RoomClient::new(&CrawlerConfig::default()).unwrap());
        let fetcher = RoomFetcher::new(client);
        
        let raw = vec![
            RoomInfo { roompath: "path1".into(), roomid: "101".into() },
            RoomInfo { roompath: "path1".into(), roomid: "101".into() },  // 完全重复
            RoomInfo { roompath: "path2".into(), roomid: "101".into() },
        ];
        
        let (merged, _stats) = fetcher.group_by_roomid_from_info(raw).unwrap();
        
        assert_eq!(merged.len(), 1);
        
        let room_101 = &merged[0];
        assert_eq!(room_101.path_count, 2);  // 去重后只有2个路径
    }
}
