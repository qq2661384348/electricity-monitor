//! 应用程序全局状态
//!
//! 管理共享资源如数据库连接池、配置等

use crate::infrastructure::DbPool;

/// 应用程序状态
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接池
    pub db_pool: DbPool,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }
}
