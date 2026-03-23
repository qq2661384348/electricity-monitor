//! RoomId内存缓存模块
//!
//! 缓存活跃房间的roomid列表，避免频繁查询数据库

use crate::errors::Result;
use crate::infrastructure::{repositories::RoomRepository, DbPool};
use std::sync::Arc;
use tokio::sync::RwLock;

/// RoomId内存缓存
///
/// 使用Arc<RwLock<Vec<i32>>>实现线程安全的读写锁
pub struct RoomIdCache {
    /// roomid缓存（Vec<i32>）
    cache: Arc<RwLock<Vec<i32>>>,
    /// 数据库连接池（用于刷新缓存）
    pool: DbPool,
}

impl RoomIdCache {
    /// 创建RoomId缓存（初始化时自动加载）
    ///
    /// # 参数
    /// - `pool`: 数据库连接池
    ///
    /// # 返回
    /// RoomIdCache实例
    ///
    /// # 错误
    /// 如果初始加载失败，返回数据库错误
    pub async fn new(pool: DbPool) -> Result<Self> {
        let cache = Arc::new(RwLock::new(Vec::new()));
        let instance = Self { cache, pool };

        // 初始化时自动加载
        instance.refresh().await?;

        Ok(instance)
    }

    /// 刷新缓存（从数据库重新加载）
    ///
    /// # 说明
    /// - 查询is_active=true的房间
    /// - 使用写锁更新缓存
    /// - 记录结构化日志
    ///
    /// # 错误
    /// 如果查询失败，返回数据库错误
    pub async fn refresh(&self) -> Result<()> {
        let repo = RoomRepository::new(self.pool.clone());
        let room_ids = repo.find_all_active_roomids().await?;

        // 获取写锁更新缓存
        let mut cache = self.cache.write().await;
        *cache = room_ids;

        tracing::info!(count = cache.len(), "RoomId缓存已刷新");

        Ok(())
    }

    /// 获取所有roomid
    ///
    /// # 返回
    /// roomid列表的克隆（Vec<i32>）
    ///
    /// # 说明
    /// - 使用读锁，支持并发读取
    /// - 返回克隆，避免锁持有时间过长
    pub async fn get_all(&self) -> Vec<i32> {
        self.cache.read().await.clone()
    }

    /// 获取缓存大小
    ///
    /// # 返回
    /// 缓存中roomid的数量
    pub async fn len(&self) -> usize {
        self.cache.read().await.len()
    }

    /// 检查缓存是否为空
    ///
    /// # 返回
    /// true表示缓存为空
    pub async fn is_empty(&self) -> bool {
        self.cache.read().await.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这是集成测试，需要数据库连接
    // 在CI/CD环境中，应该mock RoomRepository

    #[test]
    fn test_roomid_cache_struct() {
        // 测试结构体可以正常构造（编译时验证）
        // RoomIdCache { cache: Arc<RwLock<Vec<i32>>>, pool: DbPool }
        // Arc: 8 bytes, DbPool (Arc内部): 8 bytes = 16 bytes total
        assert_eq!(std::mem::size_of::<RoomIdCache>(), 16);
    }
}
