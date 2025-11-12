//! 房间数据爬取器
//!
//! 负责从外部API获取房间数据，并实现1:N合并逻辑

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;

use super::client::RoomClient;
use super::models::{MergeStatistics, RawRoomInfo, RoomData};
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
        tracing::info!("开始获取房间数据...");
        
        // 1. 获取原始扁平数据
        let raw = self.fetch_all_raw().await
            .context("获取原始房间数据失败")?;
        
        tracing::info!("获取到{}条原始记录", raw.len());
        
        // 2. 按roomid合并（1:N场景处理）
        let (merged, stats) = self.group_by_roomid(raw)
            .context("合并房间数据失败")?;
        
        // 3. 输出统计信息
        stats.log();
        
        // 4. 检查是否有数据
        if merged.is_empty() {
            tracing::warn!("未获取到任何有效房间数据");
        }
        
        tracing::info!("房间数据获取完成，共{}个有效roomid", merged.len());
        
        Ok(merged)
    }
    
    /// 获取原始扁平数据（内部方法）
    /// 
    /// 从API获取未合并的原始数据，一条记录对应一个roompath
    async fn fetch_all_raw(&self) -> Result<Vec<RawRoomInfo>> {
        // 1. 发送HTTP请求
        let json_str = self.client.fetch_room_tree()
            .await
            .context("HTTP请求失败")?;
        
        // 2. 解析JSON
        let room_tuples = parser::parse_room_tree(&json_str)
            .context("JSON解析失败")?;
        
        // 3. 转换为RawRoomInfo
        let raw: Vec<RawRoomInfo> = room_tuples
            .into_iter()
            .map(|(roompath, roomid)| RawRoomInfo { roompath, roomid })
            .collect();
        
        Ok(raw)
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
    fn group_by_roomid(&self, raw: Vec<RawRoomInfo>) -> Result<(Vec<RoomData>, MergeStatistics)> {
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
            RawRoomInfo { roompath: "path1".into(), roomid: "101".into() },
            RawRoomInfo { roompath: "path2".into(), roomid: "102".into() },
            RawRoomInfo { roompath: "path3".into(), roomid: "103".into() },
        ];
        
        let (merged, stats) = fetcher.group_by_roomid(raw).unwrap();
        
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
            RawRoomInfo { roompath: "path1".into(), roomid: "101".into() },
            RawRoomInfo { roompath: "path2".into(), roomid: "101".into() },  // 同一roomid
            RawRoomInfo { roompath: "path3".into(), roomid: "102".into() },
        ];
        
        let (merged, stats) = fetcher.group_by_roomid(raw).unwrap();
        
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
            RawRoomInfo { roompath: "path1".into(), roomid: "101".into() },
            RawRoomInfo { roompath: "path2".into(), roomid: "invalid".into() },  // 无效roomid
            RawRoomInfo { roompath: "path3".into(), roomid: "abc123".into() },  // 无效roomid
        ];
        
        let (merged, stats) = fetcher.group_by_roomid(raw).unwrap();
        
        assert_eq!(merged.len(), 1);  // 只有1个有效roomid
        assert_eq!(stats.parse_error_count, 2);  // 2个转换失败
    }

    #[test]
    fn test_group_by_roomid_deduplication() {
        let client = Arc::new(RoomClient::new(&CrawlerConfig::default()).unwrap());
        let fetcher = RoomFetcher::new(client);
        
        let raw = vec![
            RawRoomInfo { roompath: "path1".into(), roomid: "101".into() },
            RawRoomInfo { roompath: "path1".into(), roomid: "101".into() },  // 完全重复
            RawRoomInfo { roompath: "path2".into(), roomid: "101".into() },
        ];
        
        let (merged, stats) = fetcher.group_by_roomid(raw).unwrap();
        
        assert_eq!(merged.len(), 1);
        
        let room_101 = &merged[0];
        assert_eq!(room_101.path_count, 2);  // 去重后只有2个路径
    }
}
