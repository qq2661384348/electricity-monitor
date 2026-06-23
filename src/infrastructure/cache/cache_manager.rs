//! 统一缓存管理器
//!
//! 管理所有实体缓存，提供统一的接口和监控

use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::electricity_loader::ElectricityCacheLoader;
use super::entity_cache::{CacheConfig, DataLoader, EntityCache};
use crate::domain::models::{Room, User, UserRoomBinding};
use crate::errors::Result;
use crate::infrastructure::{
    repositories::{RoomRepository, UserRepository, UserRoomBindingRepository},
    DbPool, RedisPool,
};

/// Room数据加载器
struct RoomLoader {
    repository: RoomRepository,
}

#[async_trait]
impl DataLoader<i64, Room> for RoomLoader {
    async fn load(&self, key: &i64) -> Result<Option<Room>> {
        self.repository.find_by_roomid(*key).await
    }

    async fn load_batch(&self, keys: &[i64]) -> Result<Vec<(i64, Room)>> {
        let rooms = self.repository.find_by_roomids(keys).await?;
        Ok(rooms.into_iter().map(|r| (r.roomid, r)).collect())
    }
}

/// User数据加载器
struct UserLoader {
    repository: UserRepository,
}

#[async_trait]
impl DataLoader<Uuid, User> for UserLoader {
    async fn load(&self, key: &Uuid) -> Result<Option<User>> {
        self.repository.find_by_id(*key).await
    }

    async fn load_batch(&self, keys: &[Uuid]) -> Result<Vec<(Uuid, User)>> {
        let users = self.repository.find_by_ids(keys).await?;
        Ok(users.into_iter().map(|u| (u.id, u)).collect())
    }
}

/// Binding数据加载器
struct BindingLoader {
    repository: UserRoomBindingRepository,
}

#[async_trait]
impl DataLoader<i64, Vec<UserRoomBinding>> for BindingLoader {
    async fn load(&self, key: &i64) -> Result<Option<Vec<UserRoomBinding>>> {
        let bindings = self.repository.find_active_bindings_by_roomid(*key).await?;
        Ok(if bindings.is_empty() {
            None
        } else {
            Some(bindings)
        })
    }

    async fn load_batch(&self, keys: &[i64]) -> Result<Vec<(i64, Vec<UserRoomBinding>)>> {
        let all_bindings = self
            .repository
            .find_active_bindings_by_roomids(keys)
            .await?;

        // 按roomid分组
        let grouped: DashMap<i64, Vec<UserRoomBinding>> = DashMap::new();
        for binding in all_bindings {
            grouped.entry(binding.roomid).or_default().push(binding);
        }

        Ok(grouped.into_iter().collect())
    }
}

/// 缓存管理器配置
#[derive(Debug, Clone)]
pub struct CacheManagerConfig {
    /// Room缓存配置
    pub room_cache: CacheConfig,
    /// User缓存配置
    pub user_cache: CacheConfig,
    /// Binding缓存配置
    pub binding_cache: CacheConfig,
    /// 电费数据缓存配置
    pub electricity_cache: CacheConfig,
}

impl Default for CacheManagerConfig {
    fn default() -> Self {
        Self {
            room_cache: CacheConfig {
                l1_max_capacity: 10_000,
                l1_ttl_seconds: 300,  // 5分钟
                l1_tti_seconds: 60,   // 1分钟
                l2_ttl_seconds: 1800, // 30分钟
                enable_l2: true,
                enable_warming: true,
                null_cache_seconds: 60,
            },
            user_cache: CacheConfig {
                l1_max_capacity: 5_000,
                l1_ttl_seconds: 600,  // 10分钟
                l1_tti_seconds: 120,  // 2分钟
                l2_ttl_seconds: 3600, // 1小时
                enable_l2: true,
                enable_warming: false,
                null_cache_seconds: 30,
            },
            binding_cache: CacheConfig {
                l1_max_capacity: 5_000,
                l1_ttl_seconds: 180, // 3分钟
                l1_tti_seconds: 60,  // 1分钟
                l2_ttl_seconds: 900, // 15分钟
                enable_l2: true,
                enable_warming: true,
                null_cache_seconds: 30,
            },
            electricity_cache: CacheConfig {
                l1_max_capacity: 20_000,
                l1_ttl_seconds: 60,  // 1分钟（电费数据更新频繁）
                l1_tti_seconds: 30,  // 30秒
                l2_ttl_seconds: 300, // 5分钟
                enable_l2: true,
                enable_warming: false,
                null_cache_seconds: 0, // 不缓存空值
            },
        }
    }
}

/// 统一缓存管理器
pub struct CacheManager {
    /// Room缓存
    room_cache: Arc<EntityCache<i64, Room>>,
    /// User缓存
    user_cache: Arc<EntityCache<Uuid, User>>,
    /// Binding缓存
    binding_cache: Arc<EntityCache<i64, Vec<UserRoomBinding>>>,
    /// 电费数据缓存
    electricity_cache: Arc<EntityCache<i64, f32>>,
    /// 数据加载器
    room_loader: Arc<RoomLoader>,
    user_loader: Arc<UserLoader>,
    binding_loader: Arc<BindingLoader>,
    electricity_loader: Arc<ElectricityCacheLoader>,
}

impl CacheManager {
    /// 创建缓存管理器
    pub fn new(config: CacheManagerConfig, db_pool: DbPool, redis_pool: Option<RedisPool>) -> Self {
        // 创建缓存实例
        let room_cache = Arc::new(EntityCache::new(
            "cache:room",
            config.room_cache,
            redis_pool.clone(),
        ));

        let user_cache = Arc::new(EntityCache::new(
            "cache:user",
            config.user_cache,
            redis_pool.clone(),
        ));

        let binding_cache = Arc::new(EntityCache::new(
            "cache:binding",
            config.binding_cache,
            redis_pool.clone(),
        ));

        let electricity_cache = Arc::new(EntityCache::new(
            "cache:electricity",
            config.electricity_cache,
            redis_pool,
        ));

        // 创建数据加载器
        let room_loader = Arc::new(RoomLoader {
            repository: RoomRepository::new(db_pool.clone()),
        });

        let user_loader = Arc::new(UserLoader {
            repository: UserRepository::new(db_pool.clone()),
        });

        let binding_loader = Arc::new(BindingLoader {
            repository: UserRoomBindingRepository::new(db_pool.clone()),
        });

        let electricity_loader =
            Arc::new(ElectricityCacheLoader::new(RoomRepository::new(db_pool)));

        Self {
            room_cache,
            user_cache,
            binding_cache,
            electricity_cache,
            room_loader,
            user_loader,
            binding_loader,
            electricity_loader,
        }
    }

    /// 获取房间（带缓存）
    pub async fn get_room(&self, roomid: i64) -> Result<Option<Room>> {
        self.room_cache
            .get_or_load(roomid, self.room_loader.as_ref())
            .await
    }

    /// 批量获取房间
    pub async fn get_rooms(&self, roomids: Vec<i64>) -> Result<Vec<(i64, Option<Room>)>> {
        self.room_cache
            .get_batch(roomids, self.room_loader.as_ref())
            .await
    }

    /// 获取用户（带缓存）
    pub async fn get_user(&self, user_id: Uuid) -> Result<Option<User>> {
        self.user_cache
            .get_or_load(user_id, self.user_loader.as_ref())
            .await
    }

    /// 批量获取用户
    pub async fn get_users(&self, user_ids: Vec<Uuid>) -> Result<Vec<(Uuid, Option<User>)>> {
        self.user_cache
            .get_batch(user_ids, self.user_loader.as_ref())
            .await
    }

    /// 获取房间绑定（带缓存）
    pub async fn get_room_bindings(&self, roomid: i64) -> Result<Option<Vec<UserRoomBinding>>> {
        self.binding_cache
            .get_or_load(roomid, self.binding_loader.as_ref())
            .await
    }

    /// 批量获取房间绑定
    pub async fn get_rooms_bindings(
        &self,
        roomids: Vec<i64>,
    ) -> Result<Vec<(i64, Option<Vec<UserRoomBinding>>)>> {
        self.binding_cache
            .get_batch(roomids, self.binding_loader.as_ref())
            .await
    }

    /// 获取电费（带缓存）
    pub async fn get_electricity(&self, roomid: i64) -> Result<Option<f32>> {
        self.electricity_cache
            .get_or_load(roomid, self.electricity_loader.as_ref())
            .await
    }

    /// 批量获取电费
    pub async fn get_electricity_batch(
        &self,
        roomids: Vec<i64>,
    ) -> Result<Vec<(i64, Option<f32>)>> {
        self.electricity_cache
            .get_batch(roomids, self.electricity_loader.as_ref())
            .await
    }

    /// 设置电费缓存（用于更新后）
    pub async fn set_electricity(&self, roomid: i64, electricity_fee: f32) -> Result<()> {
        self.electricity_cache.set(roomid, electricity_fee).await
    }

    /// 批量设置电费缓存
    pub async fn set_electricity_batch(&self, data: Vec<(i64, f32)>) -> Result<()> {
        for (roomid, electricity_fee) in data {
            self.electricity_cache.set(roomid, electricity_fee).await?;
        }
        Ok(())
    }

    /// 使电费缓存失效
    pub async fn invalidate_electricity(&self, roomid: i64) -> Result<()> {
        self.electricity_cache.invalidate(&roomid).await
    }

    /// 使房间缓存失效
    pub async fn invalidate_room(&self, roomid: i64) -> Result<()> {
        self.room_cache.invalidate(&roomid).await?;
        // 同时使相关电费缓存失效
        self.electricity_cache.invalidate(&roomid).await?;
        Ok(())
    }

    /// 使用户缓存失效
    pub async fn invalidate_user(&self, user_id: Uuid) -> Result<()> {
        self.user_cache.invalidate(&user_id).await
    }

    /// 使绑定缓存失效
    pub async fn invalidate_binding(&self, roomid: i64) -> Result<()> {
        self.binding_cache.invalidate(&roomid).await
    }

    /// 预热缓存（用于启动时）
    pub async fn warm_cache(&self, roomids: Vec<i64>) -> Result<()> {
        tracing::info!("开始预热缓存，房间数: {}", roomids.len());

        // 并发预热
        let (rooms, bindings) = tokio::join!(
            self.get_rooms(roomids.clone()),
            self.get_rooms_bindings(roomids)
        );

        rooms?;
        bindings?;

        tracing::info!("缓存预热完成");
        Ok(())
    }

    /// 获取缓存统计
    pub fn stats(&self) -> CacheManagerStats {
        CacheManagerStats {
            room_stats: self.room_cache.stats(),
            user_stats: self.user_cache.stats(),
            binding_stats: self.binding_cache.stats(),
            electricity_stats: self.electricity_cache.stats(),
        }
    }

    /// 清空所有缓存
    pub async fn clear_all(&self) -> Result<()> {
        tokio::try_join!(
            self.room_cache.invalidate_all(),
            self.user_cache.invalidate_all(),
            self.binding_cache.invalidate_all(),
            self.electricity_cache.invalidate_all()
        )?;
        Ok(())
    }
}

/// 缓存管理器统计
#[derive(Debug, Clone)]
pub struct CacheManagerStats {
    pub room_stats: super::entity_cache::CacheStats,
    pub user_stats: super::entity_cache::CacheStats,
    pub binding_stats: super::entity_cache::CacheStats,
    pub electricity_stats: super::entity_cache::CacheStats,
}

impl CacheManagerStats {
    /// 打印统计信息（简化版）
    pub fn log_stats(&self) {
        tracing::info!(
            "缓存统计 - Room: L1命中={} L1未命中={} L2命中={} L2未命中={} 加载={}",
            self.room_stats.l1_hits,
            self.room_stats.l1_misses,
            self.room_stats.l2_hits,
            self.room_stats.l2_misses,
            self.room_stats.loads
        );

        tracing::info!(
            "缓存统计 - User: L1命中={} L1未命中={} L2命中={} L2未命中={} 加载={}",
            self.user_stats.l1_hits,
            self.user_stats.l1_misses,
            self.user_stats.l2_hits,
            self.user_stats.l2_misses,
            self.user_stats.loads
        );

        tracing::info!(
            "缓存统计 - Binding: L1命中={} L1未命中={} L2命中={} L2未命中={} 加载={}",
            self.binding_stats.l1_hits,
            self.binding_stats.l1_misses,
            self.binding_stats.l2_hits,
            self.binding_stats.l2_misses,
            self.binding_stats.loads
        );

        tracing::info!(
            "缓存统计 - Electricity: L1命中={} L1未命中={} L2命中={} L2未命中={} 加载={}",
            self.electricity_stats.l1_hits,
            self.electricity_stats.l1_misses,
            self.electricity_stats.l2_hits,
            self.electricity_stats.l2_misses,
            self.electricity_stats.loads
        );

        let total_hits = self.room_stats.l1_hits
            + self.room_stats.l2_hits
            + self.user_stats.l1_hits
            + self.user_stats.l2_hits
            + self.binding_stats.l1_hits
            + self.binding_stats.l2_hits
            + self.electricity_stats.l1_hits
            + self.electricity_stats.l2_hits;

        let total_misses = self.room_stats.l1_misses
            + self.room_stats.l2_misses
            + self.user_stats.l1_misses
            + self.user_stats.l2_misses
            + self.binding_stats.l1_misses
            + self.binding_stats.l2_misses
            + self.electricity_stats.l1_misses
            + self.electricity_stats.l2_misses;

        let hit_rate = if total_hits + total_misses > 0 {
            total_hits as f64 / (total_hits + total_misses) as f64
        } else {
            0.0
        };

        tracing::info!(
            "缓存汇总 - 总命中: {} 总未命中: {} 命中率: {:.1}%",
            total_hits,
            total_misses,
            hit_rate * 100.0
        );
    }
}
