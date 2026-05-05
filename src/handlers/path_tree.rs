//! 房间路径树 API 处理器
//!
//! 提供逐层查询房间路径的接口

use axum::{
    extract::{Query, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};

use crate::errors::Result;
use crate::middleware::auth::UserContext;
use crate::modules::room::{application::RoomAccessUseCase, domain::RoomActor};
use crate::state::AppState;
use crate::utils::hash::calculate_roompath_hash;

/// 查询路径子节点请求
#[derive(Debug, Deserialize)]
pub struct QueryPathRequest {
    /// 父路径（空字符串表示查询根节点）
    #[serde(default)]
    pub parent: String,
}

/// 路径子节点响应
#[derive(Debug, Serialize)]
pub struct PathChildrenResponse {
    /// 子节点列表
    pub children: Vec<PathChild>,

    /// 当前层级（0=校区，1=建筑，2=楼层，3=房间）
    pub current_level: usize,

    /// 总数
    pub total_count: usize,
}

/// 路径子节点
#[derive(Debug, Serialize)]
pub struct PathChild {
    /// 节点名称
    pub name: String,

    /// 是否为叶子节点（房间）
    pub is_leaf: bool,

    /// 该节点下的房间总数
    pub room_count: usize,

    /// 叶子节点绑定用的最小房间标识；非叶子节点不返回。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roomid: Option<i32>,
}

/// 查询路径的子节点
///
/// GET /api/rooms/path-tree?parent={encoded_path}
///
/// # 示例
/// - `GET /api/rooms/path-tree` - 查询校区列表
/// - `GET /api/rooms/path-tree?parent=箭盘校区` - 查询建筑列表
/// - `GET /api/rooms/path-tree?parent=箭盘校区/北区12栋` - 查询楼层列表
pub async fn query_path_tree(
    State(state): State<AppState>,
    Query(params): Query<QueryPathRequest>,
) -> Result<Json<PathChildrenResponse>> {
    tracing::debug!("查询路径树: parent={}", params.parent);

    let tree = state.room_path_tree.read().await;
    let children = tree.query_children(&params.parent).await?;

    // 计算当前层级（根据分隔符数量）
    let current_level = if params.parent.is_empty() {
        0
    } else {
        params.parent.matches('/').count() + 1
    };

    let total_count = children.len();

    // 转换为响应格式
    let children_response: Vec<PathChild> = children
        .into_iter()
        .map(|child| PathChild {
            name: child.name,
            is_leaf: child.is_leaf,
            room_count: child.room_count,
            roomid: child.roomid,
        })
        .collect();

    Ok(Json(PathChildrenResponse {
        children: children_response,
        current_level,
        total_count,
    }))
}

/// 根据完整路径查询房间
#[derive(Debug, Deserialize)]
pub struct QueryByPathRequest {
    /// 完整路径（如 "箭盘校区/北区12栋/三楼/B12313"）
    pub path: String,
}

/// 根据路径查询房间响应
#[derive(Debug, Serialize)]
pub struct RoomByPathResponse {
    /// 房间ID
    pub roomid: i32,

    /// 房间名称
    pub room_name: String,

    /// 当前电费
    pub electricity_fee: f32,

    /// 电费阈值
    pub threshold: f32,

    /// 主要路径
    pub primary_roompath: String,
}

/// 根据完整路径查询房间
///
/// GET /api/rooms/by-path?path={encoded_path}
///
/// # 示例
/// `GET /api/rooms/by-path?path=箭盘校区/北区12栋/三楼/B12313`
///
/// # 返回
/// - 200: 房间信息
/// - 404: 路径不存在
pub async fn get_room_by_path(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Query(params): Query<QueryByPathRequest>,
) -> Result<Json<RoomByPathResponse>> {
    tracing::debug!("根据路径查询房间: path={}", params.path);

    let actor = RoomActor::from_user_context(&user_ctx)?;
    let use_case = RoomAccessUseCase::from_state(&state);
    let room = use_case.get_room_by_path(&actor, &params.path).await?;

    Ok(Json(RoomByPathResponse {
        roomid: room.roomid,
        room_name: room.room_name,
        electricity_fee: room.electricity_fee,
        threshold: room.threshold,
        primary_roompath: room.primary_roompath,
    }))
}

/// 根据路径哈希查询房间（高性能版本）
#[derive(Debug, Deserialize)]
pub struct QueryByHashRequest {
    /// 路径哈希值
    pub hash: i64,

    /// 完整路径（用于验证，防止哈希冲突）
    pub path: String,
}

/// 根据路径哈希查询房间（高性能版本）
///
/// GET /api/rooms/by-hash?hash={hash}&path={encoded_path}
///
/// # 性能优势
/// 使用预先计算的哈希值，查询复杂度为 O(1)
///
/// # 返回
/// - 200: 房间信息
/// - 404: 哈希或路径不匹配
pub async fn get_room_by_hash(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Query(params): Query<QueryByHashRequest>,
) -> Result<Json<RoomByPathResponse>> {
    tracing::debug!(
        "根据哈希查询房间: hash={}, path={}",
        params.hash,
        params.path
    );

    let actor = RoomActor::from_user_context(&user_ctx)?;
    let use_case = RoomAccessUseCase::from_state(&state);
    let room = use_case
        .get_room_by_hash(&actor, params.hash, &params.path)
        .await?;

    Ok(Json(RoomByPathResponse {
        roomid: room.roomid,
        room_name: room.room_name,
        electricity_fee: room.electricity_fee,
        threshold: room.threshold,
        primary_roompath: room.primary_roompath,
    }))
}

/// 计算路径哈希值（辅助接口）
#[derive(Debug, Deserialize)]
pub struct CalculateHashRequest {
    /// 路径字符串
    pub path: String,
}

/// 哈希计算响应
#[derive(Debug, Serialize)]
pub struct HashResponse {
    /// 路径
    pub path: String,

    /// 计算出的哈希值
    pub hash: i64,
}

/// 计算路径哈希值
///
/// GET /api/rooms/calculate-hash?path={encoded_path}
///
/// # 用途
/// 前端可以预先计算哈希值，用于高性能查询
pub async fn calculate_path_hash(
    Query(params): Query<CalculateHashRequest>,
) -> Result<Json<HashResponse>> {
    let hash = calculate_roompath_hash(&params.path);

    Ok(Json(HashResponse {
        path: params.path,
        hash,
    }))
}
