//! 电费数据缓存加载器
//!
//! 实现DataLoader trait，支持从数据库加载电费数据到缓存
//! 特性：
//! - 单个查询优化
//! - 批量查询支持
//! - 自动缓存失效

use async_trait::async_trait;
use std::collections::HashMap;

use crate::errors::Result;
use crate::infrastructure::repositories::RoomRepository;

use super::entity_cache::DataLoader;

/// 电费数据加载器
///
/// 负责从数据库加载电费数据，供EntityCache使用
pub struct ElectricityCacheLoader {
    /// Room仓储
    repository: RoomRepository,
}

impl ElectricityCacheLoader {
    /// 创建新的加载器实例
    pub fn new(repository: RoomRepository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl DataLoader<i32, f32> for ElectricityCacheLoader {
    /// 加载单个房间的电费
    ///
    /// # 参数
    /// - `roomid`: 房间ID
    ///
    /// # 返回
    /// - `Some(f32)`: 电费值
    /// - `None`: 房间不存在
    async fn load(&self, roomid: &i32) -> Result<Option<f32>> {
        tracing::debug!("从数据库加载电费: roomid={}", roomid);

        // 查询房间信息
        match self.repository.find_by_roomid(*roomid).await? {
            Some(room) => {
                tracing::debug!(
                    "加载电费成功: roomid={}, electricity_fee={}",
                    roomid,
                    room.electricity_fee
                );
                Ok(Some(room.electricity_fee))
            }
            None => {
                tracing::debug!("房间不存在: roomid={}", roomid);
                Ok(None)
            }
        }
    }

    /// 批量加载多个房间的电费
    ///
    /// # 参数
    /// - `roomids`: 房间ID列表
    ///
    /// # 返回
    /// Vec<(roomid, electricity_fee)>
    ///
    /// # 性能优化
    /// - 单次数据库查询
    /// - 使用IN子句批量查询
    /// - 最大1000个ID分批处理
    async fn load_batch(&self, roomids: &[i32]) -> Result<Vec<(i32, f32)>> {
        if roomids.is_empty() {
            return Ok(Vec::new());
        }

        tracing::debug!("批量加载电费: {} 个房间", roomids.len());

        // 使用find_by_roomids批量查询
        let rooms = self.repository.find_by_roomids(roomids).await?;

        // 转换为(roomid, electricity_fee)元组
        let results: Vec<(i32, f32)> = rooms
            .into_iter()
            .map(|room| (room.roomid, room.electricity_fee))
            .collect();

        tracing::debug!("批量加载完成: 返回 {} 条数据", results.len());

        // 记录未找到的roomid（用于调试）
        if results.len() < roomids.len() {
            let found_ids: HashMap<i32, ()> = results.iter().map(|(id, _)| (*id, ())).collect();

            let missing_ids: Vec<i32> = roomids
                .iter()
                .filter(|id| !found_ids.contains_key(id))
                .copied()
                .collect();

            if !missing_ids.is_empty() {
                tracing::debug!("未找到的房间ID: {:?}", missing_ids);
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {

    // 注意：这些是集成测试，需要数据库连接
    // 在CI/CD环境中应该使用mock

    #[test]
    fn test_loader_creation() {
        // 验证加载器可以正确创建
        // 实际测试需要mock RoomRepository
    }
}
