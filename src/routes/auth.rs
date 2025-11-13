//! 认证路由
//! 
//! 定义认证相关的API路由

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::handlers::auth;
use crate::middleware::auth::auth_middleware;
use crate::state::AppState;

/// 创建认证路由
pub fn routes() -> Router<AppState> {
    // 公开路由（无需认证）
    let public_routes = Router::new()
        .route("/auth/send-verification-code", post(auth::send_verification_code))
        .route("/auth/verify-and-login", post(auth::verify_and_login))
        .route("/auth/refresh", post(auth::refresh_token));

    // 受保护路由（需要JWT认证）
    let protected_routes = Router::new()
        .route("/auth/me", get(auth::get_current_user))
        .route_layer(middleware::from_fn(auth_middleware));

    // 合并路由
    public_routes.merge(protected_routes)
}
