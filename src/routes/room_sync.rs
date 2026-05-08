//! 房间同步API路由
//!
//! 所有端点仅限管理员访问

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::handlers::room_sync;
use crate::middleware::auth::{auth_middleware, require_admin};
use crate::state::AppState;

/// 创建房间同步相关路由
///
/// # 权限要求
/// 所有端点需要管理员权限
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        // 手动触发同步
        .route("/rooms/sync", post(room_sync::trigger_sync))
        // 查询同步状态
        .route(
            "/rooms/sync/status/{job_id}",
            get(room_sync::get_sync_status),
        )
        // 查询同步历史
        .route("/rooms/sync/history", get(room_sync::get_sync_history))
        // 查询房间所有路径
        .route("/rooms/{roomid}/paths", get(room_sync::get_room_paths))
        // 注意：route_layer从下往上执行
        // require_admin 放在下面（后执行，此时 Actor 已注入）
        .route_layer(middleware::from_fn(require_admin))
        // auth_middleware 放在上面（先执行，注入 Actor / UserContext）
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}
