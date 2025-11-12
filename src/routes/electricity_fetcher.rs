//! 电费获取路由

use axum::{routing::{get, post}, Router};

use crate::handlers;
use crate::state::AppState;

/// 创建电费获取路由
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/fetch", post(handlers::trigger_fetch))
        .route("/refresh-cache", post(handlers::refresh_cache))
        .route("/status", get(handlers::get_status))
}
