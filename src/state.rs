//! 应用程序全局状态
//!
//! 管理共享资源如数据库连接池、配置等

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::models::Room;
use crate::domain::services::{ElectricityFetcherService, RateLimiter, RoomPathTree};
use crate::infrastructure::{email::EmailDelivery, CacheManager, DbPool, RedisPool};

/// 应用程序状态
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接池
    pub db_pool: DbPool,

    /// Redis连接池
    pub redis_pool: RedisPool,

    /// 限流器
    pub rate_limiter: Arc<RateLimiter>,

    /// 电费获取服务（可选）
    pub electricity_fetcher_service: Option<Arc<ElectricityFetcherService>>,

    /// 需要通知的房间缓存（避免N+1查询）
    pub flagged_rooms_cache: Arc<RwLock<Vec<Room>>>,

    /// 房间路径树（用于逐层查询）
    pub room_path_tree: Arc<RwLock<RoomPathTree>>,

    /// 统一缓存管理器
    pub cache_manager: Arc<CacheManager>,

    /// 邮件发送器。未配置 SMTP 时为 None，邮箱登录和邮箱通知会明确拒绝发送。
    pub email_sender: Option<Arc<dyn EmailDelivery>>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(
        db_pool: DbPool,
        redis_pool: RedisPool,
        rate_limiter: Arc<RateLimiter>,
        electricity_fetcher_service: Option<Arc<ElectricityFetcherService>>,
        cache_manager: Arc<CacheManager>,
    ) -> Self {
        Self {
            db_pool,
            redis_pool,
            rate_limiter,
            electricity_fetcher_service,
            flagged_rooms_cache: Arc::new(RwLock::new(Vec::new())),
            room_path_tree: Arc::new(RwLock::new(RoomPathTree::new())),
            cache_manager,
            email_sender: None,
        }
    }

    pub fn with_email_sender(mut self, email_sender: Option<Arc<dyn EmailDelivery>>) -> Self {
        self.email_sender = email_sender;
        self
    }

    /// 更新路径树（由同步服务调用）
    pub async fn update_path_tree(&self, tree: RoomPathTree) {
        let mut current_tree = self.room_path_tree.write().await;
        *current_tree = tree;
        tracing::info!("路径树已更新");
    }
}
