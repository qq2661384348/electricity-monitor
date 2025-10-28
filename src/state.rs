//! 应用程序全局状态
//!
//! 管理共享资源如数据库连接池、配置等

use std::sync::Arc;

use crate::domain::services::RateLimiter;
use crate::infrastructure::{DbPool, RedisPool};

/// 应用程序状态
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接池
    pub db_pool: DbPool,
    
    /// Redis连接池
    pub redis_pool: RedisPool,
    
    /// 限流器
    pub rate_limiter: Arc<RateLimiter>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(db_pool: DbPool, redis_pool: RedisPool, rate_limiter: Arc<RateLimiter>) -> Self {
        Self {
            db_pool,
            redis_pool,
            rate_limiter,
        }
    }
}
