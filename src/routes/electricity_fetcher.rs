//! 电费获取路由
//!
//! 所有端点仅限管理员访问

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::handlers;
use crate::middleware::auth::{auth_middleware, require_admin};
use crate::state::AppState;

/// 创建电费获取路由
///
/// # 权限要求
/// 所有端点需要管理员权限
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/fetch", post(handlers::trigger_fetch))
        .route("/refresh-cache", post(handlers::refresh_cache))
        .route("/status", get(handlers::get_status))
        // 注意：route_layer从下往上执行
        .route_layer(middleware::from_fn(require_admin))
        .route_layer(middleware::from_fn(auth_middleware))
}
