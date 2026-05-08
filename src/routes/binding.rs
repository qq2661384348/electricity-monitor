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
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/bindings", post(binding::create_binding))
        .route("/bindings", get(binding::list_bindings))
        .route("/bindings/{id}", get(binding::get_binding))
        .route(
            "/bindings/{id}/notification",
            put(binding::update_notification),
        )
        .route("/bindings/{id}", delete(binding::delete_binding))
        // 注意：route_layer是从下往上执行，所以require_user要放在下面（后执行）
        .route_layer(middleware::from_fn(require_user))
        // auth_middleware放在上面（先执行）
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}
