//! 通知服务内存缓存
//!
//! 使用LRU缓存减少数据库查询，提升通知服务性能
//!
//! # 缓存策略
//! - **用户缓存**: 1000个用户，TTL=300秒
//! - **绑定缓存**: 500个房间的绑定关系，TTL=300秒
//! - **LRU驱逐**: 缓存满时自动驱逐最久未使用的项
//!
//! # 线程安全
//! 使用`Arc<RwLock>`实现线程安全的并发访问

use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::models::{User, UserRoomBinding};

/// 缓存项（带过期时间）
#[derive(Clone)]
struct CachedItem<T> {
    /// 缓存的数据
    data: T,
    /// 缓存创建时间
    created_at: Instant,
}

impl<T> CachedItem<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            created_at: Instant::now(),
        }
    }

    /// 检查是否过期
    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

/// 通知缓存统计信息
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// 用户缓存命中次数
    pub user_hits: u64,
    /// 用户缓存未命中次数
    pub user_misses: u64,
    /// 绑定缓存命中次数
    pub binding_hits: u64,
    /// 绑定缓存未命中次数
    pub binding_misses: u64,
}

impl CacheStats {
    /// 计算用户缓存命中率
    pub fn user_hit_rate(&self) -> f64 {
        let total = self.user_hits + self.user_misses;
        if total == 0 {
            0.0
        } else {
            self.user_hits as f64 / total as f64
        }
    }

    /// 计算绑定缓存命中率
    pub fn binding_hit_rate(&self) -> f64 {
        let total = self.binding_hits + self.binding_misses;
        if total == 0 {
            0.0
        } else {
            self.binding_hits as f64 / total as f64
        }
    }
}

/// 通知服务缓存管理器
pub struct NotificationCache {
    /// 用户缓存 (user_id -> User)
    user_cache: Arc<RwLock<LruCache<Uuid, CachedItem<User>>>>,

    /// 绑定缓存 (roomid -> Vec<UserRoomBinding>)
    binding_cache: Arc<RwLock<LruCache<i32, CachedItem<Vec<UserRoomBinding>>>>>,

    /// 缓存过期时间（TTL）
    ttl: Duration,

    /// 统计信息
    stats: Arc<RwLock<CacheStats>>,
}

impl NotificationCache {
    /// 创建新的缓存管理器
    ///
    /// # 参数
    /// - `user_capacity`: 用户缓存容量（默认1000）
    /// - `binding_capacity`: 绑定缓存容量（默认500）
    /// - `ttl_secs`: 缓存过期时间（秒）（默认300秒=5分钟）
    pub fn new(
        user_capacity: Option<usize>,
        binding_capacity: Option<usize>,
        ttl_secs: Option<u64>,
    ) -> Self {
        let user_cap = user_capacity.unwrap_or(1000);
        let binding_cap = binding_capacity.unwrap_or(500);
        let ttl = Duration::from_secs(ttl_secs.unwrap_or(300));

        tracing::info!(
            user_capacity = user_cap,
            binding_capacity = binding_cap,
            ttl_secs = ttl.as_secs(),
            "创建通知缓存管理器"
        );

        Self {
            user_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(user_cap).unwrap(),
            ))),
            binding_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(binding_cap).unwrap(),
            ))),
            ttl,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// 获取用户（优先从缓存）
    ///
    /// # 返回
    /// - `Some(User)`: 缓存命中且未过期
    /// - `None`: 缓存未命中或已过期
    pub async fn get_user(&self, user_id: Uuid) -> Option<User> {
        let mut cache = self.user_cache.write().await;

        if let Some(cached) = cache.get(&user_id) {
            if !cached.is_expired(self.ttl) {
                // 缓存命中
                let mut stats = self.stats.write().await;
                stats.user_hits += 1;
                return Some(cached.data.clone());
            } else {
                // 缓存过期，移除
                cache.pop(&user_id);
            }
        }

        // 缓存未命中
        let mut stats = self.stats.write().await;
        stats.user_misses += 1;
        None
    }

    /// 设置用户缓存
    pub async fn set_user(&self, user: User) {
        let mut cache = self.user_cache.write().await;
        cache.put(user.id, CachedItem::new(user));
    }

    /// 批量设置用户缓存
    pub async fn set_users(&self, users: Vec<User>) {
        let mut cache = self.user_cache.write().await;
        for user in users {
            cache.put(user.id, CachedItem::new(user));
        }
    }

    /// 获取房间的绑定关系（优先从缓存）
    ///
    /// # 返回
    /// - `Some(Vec<UserRoomBinding>)`: 缓存命中且未过期
    /// - `None`: 缓存未命中或已过期
    pub async fn get_bindings(&self, roomid: i32) -> Option<Vec<UserRoomBinding>> {
        let mut cache = self.binding_cache.write().await;

        if let Some(cached) = cache.get(&roomid) {
            if !cached.is_expired(self.ttl) {
                // 缓存命中
                let mut stats = self.stats.write().await;
                stats.binding_hits += 1;
                return Some(cached.data.clone());
            } else {
                // 缓存过期，移除
                cache.pop(&roomid);
            }
        }

        // 缓存未命中
        let mut stats = self.stats.write().await;
        stats.binding_misses += 1;
        None
    }

    /// 设置绑定缓存
    pub async fn set_bindings(&self, roomid: i32, bindings: Vec<UserRoomBinding>) {
        let mut cache = self.binding_cache.write().await;
        cache.put(roomid, CachedItem::new(bindings));
    }

    /// 批量设置绑定缓存
    pub async fn set_bindings_batch(&self, bindings_map: HashMap<i32, Vec<UserRoomBinding>>) {
        let mut cache = self.binding_cache.write().await;
        for (roomid, bindings) in bindings_map {
            cache.put(roomid, CachedItem::new(bindings));
        }
    }

    /// 使用户缓存失效
    pub async fn invalidate_user(&self, user_id: Uuid) {
        let mut cache = self.user_cache.write().await;
        cache.pop(&user_id);
    }

    /// 使绑定缓存失效
    pub async fn invalidate_bindings(&self, roomid: i32) {
        let mut cache = self.binding_cache.write().await;
        cache.pop(&roomid);
    }

    /// 清空所有缓存
    pub async fn clear_all(&self) {
        let mut user_cache = self.user_cache.write().await;
        let mut binding_cache = self.binding_cache.write().await;

        user_cache.clear();
        binding_cache.clear();

        tracing::info!("已清空所有缓存");
    }

    /// 获取缓存统计信息
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();
    }

    /// 获取缓存大小信息
    pub async fn get_cache_sizes(&self) -> (usize, usize) {
        let user_cache = self.user_cache.read().await;
        let binding_cache = self.binding_cache.read().await;
        (user_cache.len(), binding_cache.len())
    }

    /// 打印缓存统计信息（用于日志）
    pub async fn log_stats(&self) {
        let stats = self.get_stats().await;
        let (user_size, binding_size) = self.get_cache_sizes().await;

        tracing::info!(
            user_hits = stats.user_hits,
            user_misses = stats.user_misses,
            user_hit_rate = format!("{:.2}%", stats.user_hit_rate() * 100.0),
            user_cache_size = user_size,
            binding_hits = stats.binding_hits,
            binding_misses = stats.binding_misses,
            binding_hit_rate = format!("{:.2}%", stats.binding_hit_rate() * 100.0),
            binding_cache_size = binding_size,
            "缓存统计信息"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_creation() {
        let cache = NotificationCache::new(Some(100), Some(50), Some(60));
        let (user_size, binding_size) = cache.get_cache_sizes().await;
        assert_eq!(user_size, 0);
        assert_eq!(binding_size, 0);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = NotificationCache::new(Some(10), Some(10), Some(60));

        // 初始状态
        let stats = cache.get_stats().await;
        assert_eq!(stats.user_hits, 0);
        assert_eq!(stats.user_misses, 0);

        // 模拟缓存未命中
        let user_id = Uuid::new_v4();
        let result = cache.get_user(user_id).await;
        assert!(result.is_none());

        let stats = cache.get_stats().await;
        assert_eq!(stats.user_misses, 1);
    }
}
