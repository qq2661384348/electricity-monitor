//! 路由定义
//!
//! 定义所有API路由

use axum::Router;
use crate::state::AppState;

pub mod api;

/// 创建应用路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .nest("/api", api::routes())
}
