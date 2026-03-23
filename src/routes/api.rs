//! API路由定义

use axum::{routing::get, Router};

use crate::handlers;
use crate::state::AppState;

/// 创建API路由
pub fn routes() -> Router<AppState> {
    Router::new()
        // 健康检查
        .route("/health", get(handlers::health_check))
        .route("/health/db", get(handlers::health_check_db))
}
