//! 房间同步API路由

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::room_sync;
use crate::state::AppState;

/// 创建房间同步相关路由
pub fn routes() -> Router<AppState> {
    Router::new()
        // 手动触发同步
        .route("/rooms/sync", post(room_sync::trigger_sync))
        // 查询同步状态
        .route("/rooms/sync/status/{job_id}", get(room_sync::get_sync_status))
        // 查询同步历史
        .route("/rooms/sync/history", get(room_sync::get_sync_history))
        // 查询房间所有路径
        .route("/rooms/{roomid}/paths", get(room_sync::get_room_paths))
}
