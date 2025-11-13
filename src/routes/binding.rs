//! 用户-房间绑定路由
//! 
//! 定义绑定管理相关的API路由

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::handlers::binding;
use crate::middleware::auth::{auth_middleware, require_user};
use crate::state::AppState;

/// 创建绑定路由
/// 
/// 所有端点都需要JWT认证
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/bindings", post(binding::create_binding))
        .route("/bindings", get(binding::list_bindings))
        .route("/bindings/{id}", get(binding::get_binding))
        .route("/bindings/{id}/notification", put(binding::update_notification))
        .route("/bindings/{id}", delete(binding::delete_binding))
        // 应用认证中间件（所有端点都需要登录）
        .route_layer(middleware::from_fn(auth_middleware))
        // 应用用户角色验证（只有user和admin可以访问）
        .route_layer(middleware::from_fn(require_user))
}
