//! 简化的缓存实现
//!
//! 基于moka实现的高性能缓存，支持TTL和自动过期

use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache as MokaCache;

use crate::domain::models::{Room, User, UserRoomBinding};
use crate::errors::Result;
use crate::infrastructure::repositories::{
    RoomRepository, UserRepository, UserRoomBindingRepository,
};
use uuid::Uuid;

/// 简化的缓存配置
#[derive(Debug, Clone)]
pub struct SimpleCacheConfig {
    /// 最大容量
    pub max_capacity: u64,
    /// TTL（秒）
    pub ttl_seconds: u64,
    /// 空闲时间（秒）
    pub tti_seconds: u64,
}

impl Default for SimpleCacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 10_000,
            ttl_seconds: 300, // 5分钟
            tti_seconds: 60,  // 1分钟
        }
    }
}

/// Room缓存
pub struct RoomCache {
    cache: MokaCache<i32, Option<Room>>,
    repository: RoomRepository,
}

impl RoomCache {
    pub fn new(repository: RoomRepository, config: SimpleCacheConfig) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(Duration::from_secs(config.ttl_seconds))
            .time_to_idle(Duration::from_secs(config.tti_seconds))
            .build();

        Self { cache, repository }
    }

    /// 获取房间（带缓存）
    pub async fn get(&self, roomid: i32) -> Result<Option<Room>> {
        // 尝试从缓存获取
        if let Some(cached) = self.cache.get(&roomid).await {
            return Ok(cached);
        }

        // 从数据库加载
        let room = self.repository.find_by_roomid(roomid).await?;

        // 更新缓存（包括空值）
        self.cache.insert(roomid, room.clone()).await;

        Ok(room)
    }

    /// 批量获取房间
    pub async fn get_batch(&self, roomids: &[i32]) -> Result<Vec<Room>> {
        let mut results = Vec::new();
        let mut missing = Vec::new();

        // 从缓存获取
        for &roomid in roomids {
            if let Some(cached) = self.cache.get(&roomid).await {
                if let Some(room) = cached {
                    results.push(room);
                }
            } else {
                missing.push(roomid);
            }
        }

        // 批量从数据库获取缺失的
        if !missing.is_empty() {
            let rooms = self.repository.find_by_roomids(&missing).await?;
            for room in &rooms {
                self.cache.insert(room.roomid, Some(room.clone())).await;
            }
            results.extend(rooms);
        }

        Ok(results)
    }

    /// 使缓存失效
    pub async fn invalidate(&self, roomid: i32) {
        self.cache.invalidate(&roomid).await;
    }

    /// 清空缓存
    pub async fn clear(&self) {
        self.cache.invalidate_all();
    }

    /// 获取缓存大小
    pub fn size(&self) -> u64 {
        self.cache.entry_count()
    }
}

/// User缓存
pub struct UserCache {
    cache: MokaCache<Uuid, Option<User>>,
    repository: UserRepository,
}

impl UserCache {
    pub fn new(repository: UserRepository, config: SimpleCacheConfig) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(Duration::from_secs(config.ttl_seconds))
            .time_to_idle(Duration::from_secs(config.tti_seconds))
            .build();

        Self { cache, repository }
    }

    /// 获取用户（带缓存）
    pub async fn get(&self, user_id: Uuid) -> Result<Option<User>> {
        // 尝试从缓存获取
        if let Some(cached) = self.cache.get(&user_id).await {
            return Ok(cached);
        }

        // 从数据库加载
        let user = self.repository.find_by_id(user_id).await?;

        // 更新缓存
        self.cache.insert(user_id, user.clone()).await;

        Ok(user)
    }

    /// 批量获取用户
    pub async fn get_batch(&self, user_ids: &[Uuid]) -> Result<Vec<User>> {
        let mut results = Vec::new();
        let mut missing = Vec::new();

        // 从缓存获取
        for &user_id in user_ids {
            if let Some(cached) = self.cache.get(&user_id).await {
                if let Some(user) = cached {
                    results.push(user);
                }
            } else {
                missing.push(user_id);
            }
        }

        // 批量从数据库获取缺失的
        if !missing.is_empty() {
            let users = self.repository.find_by_ids(&missing).await?;
            for user in &users {
                self.cache.insert(user.id, Some(user.clone())).await;
            }
            results.extend(users);
        }

        Ok(results)
    }

    /// 使缓存失效
    pub async fn invalidate(&self, user_id: Uuid) {
        self.cache.invalidate(&user_id).await;
    }

    /// 清空缓存
    pub async fn clear(&self) {
        self.cache.invalidate_all();
    }

    /// 获取缓存大小
    pub fn size(&self) -> u64 {
        self.cache.entry_count()
    }
}

/// Binding缓存
pub struct BindingCache {
    cache: MokaCache<i32, Vec<UserRoomBinding>>,
    repository: UserRoomBindingRepository,
}

impl BindingCache {
    pub fn new(repository: UserRoomBindingRepository, config: SimpleCacheConfig) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(Duration::from_secs(config.ttl_seconds))
            .time_to_idle(Duration::from_secs(config.tti_seconds))
            .build();

        Self { cache, repository }
    }

    /// 获取房间绑定（带缓存）
    pub async fn get_by_roomid(&self, roomid: i32) -> Result<Vec<UserRoomBinding>> {
        // 尝试从缓存获取
        if let Some(cached) = self.cache.get(&roomid).await {
            return Ok(cached);
        }

        // 从数据库加载
        let bindings = self
            .repository
            .find_active_bindings_by_roomid(roomid)
            .await?;

        // 更新缓存
        self.cache.insert(roomid, bindings.clone()).await;

        Ok(bindings)
    }

    /// 批量获取房间绑定
    pub async fn get_batch(&self, roomids: &[i32]) -> Result<Vec<UserRoomBinding>> {
        let mut results = Vec::new();
        let mut missing = Vec::new();

        // 从缓存获取
        for &roomid in roomids {
            if let Some(cached) = self.cache.get(&roomid).await {
                results.extend(cached);
            } else {
                missing.push(roomid);
            }
        }

        // 批量从数据库获取缺失的
        if !missing.is_empty() {
            let all_bindings = self
                .repository
                .find_active_bindings_by_roomids(&missing)
                .await?;

            // 按roomid分组并缓存
            use std::collections::HashMap;
            let mut grouped: HashMap<i32, Vec<UserRoomBinding>> = HashMap::new();
            for binding in all_bindings {
                grouped
                    .entry(binding.roomid)
                    .or_default()
                    .push(binding.clone());
                results.push(binding);
            }

            // 更新缓存
            for (roomid, bindings) in grouped {
                self.cache.insert(roomid, bindings).await;
            }
        }

        Ok(results)
    }

    /// 使缓存失效
    pub async fn invalidate(&self, roomid: i32) {
        self.cache.invalidate(&roomid).await;
    }

    /// 清空缓存
    pub async fn clear(&self) {
        self.cache.invalidate_all();
    }

    /// 获取缓存大小
    pub fn size(&self) -> u64 {
        self.cache.entry_count()
    }
}

/// 简化的统一缓存管理器
pub struct SimpleCacheManager {
    pub room_cache: Arc<RoomCache>,
    pub user_cache: Arc<UserCache>,
    pub binding_cache: Arc<BindingCache>,
}

impl SimpleCacheManager {
    pub fn new(
        room_repo: RoomRepository,
        user_repo: UserRepository,
        binding_repo: UserRoomBindingRepository,
        config: SimpleCacheConfig,
    ) -> Self {
        Self {
            room_cache: Arc::new(RoomCache::new(room_repo, config.clone())),
            user_cache: Arc::new(UserCache::new(user_repo, config.clone())),
            binding_cache: Arc::new(BindingCache::new(binding_repo, config)),
        }
    }

    /// 预热缓存
    pub async fn warm_up(&self, roomids: &[i32]) -> Result<()> {
        tracing::info!("开始预热缓存，房间数: {}", roomids.len());

        // 并发预热
        let (rooms, bindings) = tokio::join!(
            self.room_cache.get_batch(roomids),
            self.binding_cache.get_batch(roomids)
        );

        rooms?;
        let bindings = bindings?;

        // 预热用户缓存
        let mut user_ids = Vec::new();
        for binding in bindings {
            user_ids.push(binding.user_id);
        }

        if !user_ids.is_empty() {
            self.user_cache.get_batch(&user_ids).await?;
        }

        tracing::info!("缓存预热完成");
        Ok(())
    }

    /// 获取缓存统计
    pub fn stats(&self) -> SimpleCacheStats {
        SimpleCacheStats {
            room_cache_size: self.room_cache.size(),
            user_cache_size: self.user_cache.size(),
            binding_cache_size: self.binding_cache.size(),
        }
    }

    /// 清空所有缓存
    pub async fn clear_all(&self) {
        tokio::join!(
            self.room_cache.clear(),
            self.user_cache.clear(),
            self.binding_cache.clear()
        );
    }
}

/// 简单的缓存统计
#[derive(Debug, Clone)]
pub struct SimpleCacheStats {
    pub room_cache_size: u64,
    pub user_cache_size: u64,
    pub binding_cache_size: u64,
}

impl SimpleCacheStats {
    pub fn log_stats(&self) {
        tracing::info!(
            "缓存统计 - Rooms: {} Users: {} Bindings: {}",
            self.room_cache_size,
            self.user_cache_size,
            self.binding_cache_size
        );
    }
}
