//! Room路由定义
//!
//! 所有端点需要JWT认证
//! - 管理员可以访问所有房间数据
//! - 普通用户只能访问已绑定的房间

use axum::{
    middleware,
    routing::{get, put},
    Router,
};

use crate::handlers;
use crate::middleware::auth::auth_middleware;
use crate::state::AppState;

/// 创建Room路由
///
/// # 权限要求
/// - 所有端点需要JWT认证
/// - 管理员可以访问所有数据
/// - 普通用户只能访问已绑定的房间
///
/// # 注意
/// create_room, delete_room, reset_send_flag已移除
/// 这些操作仅通过后台同步服务或Repository层内部调用
pub fn routes() -> Router<AppState> {
    Router::new()
        // 路径树相关（逐层查询）
        .route("/rooms/path-tree", get(handlers::query_path_tree))
        .route("/rooms/by-path", get(handlers::get_room_by_path))
        .route("/rooms/by-hash", get(handlers::get_room_by_hash))
        .route("/rooms/calculate-hash", get(handlers::calculate_path_hash))
        // 查询所有房间（分页）
        .route("/rooms", get(handlers::list_rooms))
        // 查询需要通知的房间
        .route("/rooms/flagged", get(handlers::get_flagged_rooms))
        // 根据roomid查询
        .route(
            "/rooms/by-roomid/{roomid}",
            get(handlers::get_rooms_by_roomid),
        )
        // 获取房间详情
        .route("/rooms/{id}", get(handlers::get_room))
        // 更新阈值
        .route("/rooms/{id}/threshold", put(handlers::update_threshold))
        // 应用JWT认证中间件
        .route_layer(middleware::from_fn(auth_middleware))
}
