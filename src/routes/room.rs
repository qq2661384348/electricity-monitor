//! Room路由定义

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::handlers;
use crate::state::AppState;

/// 创建Room路由
pub fn routes() -> Router<AppState> {
    Router::new()
        // 创建房间
        .route("/rooms", post(handlers::create_room))
        // 查询所有房间（分页）
        .route("/rooms", get(handlers::list_rooms))
        // 查询需要通知的房间
        .route("/rooms/flagged", get(handlers::get_flagged_rooms))
        // 根据roomid查询
        .route("/rooms/by-roomid/:roomid", get(handlers::get_rooms_by_roomid))
        // 获取房间详情
        .route("/rooms/:id", get(handlers::get_room))
        // 删除房间
        .route("/rooms/:id", delete(handlers::delete_room))
        // 更新阈值
        .route("/rooms/:id/threshold", put(handlers::update_threshold))
        // 重置send_flag
        .route("/rooms/:id/reset-flag", post(handlers::reset_send_flag))
}
