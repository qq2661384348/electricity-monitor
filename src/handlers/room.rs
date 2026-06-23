//! Room处理器
//!
//! 处理房间相关的HTTP请求

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::models::NewRoom;
use crate::errors::{AppError, Result};
use crate::middleware::auth::UserContext;
use crate::modules::room::{application::RoomAccessUseCase, domain::RoomActor};
use crate::state::AppState;
use crate::utils::hash::calculate_roompath_hash;
use crate::utils::roomid;

const MIN_PAGE_LIMIT: i64 = 1;
const MAX_PAGE_LIMIT: i64 = 100;
const MAX_PAGE_OFFSET: i64 = 10_000;

/// 创建房间请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateRoomRequest {
    /// 房间ID
    #[serde(deserialize_with = "roomid::deserialize")]
    pub roomid: i64,

    /// 电费阈值
    #[validate(range(min = 0.0, message = "阈值不能为负数"))]
    pub threshold: f32,

    /// 房间名称
    #[validate(length(min = 1, max = 64, message = "房间名称长度必须在1-64字符之间"))]
    pub room_name: String,

    /// 初始电费（可选，默认0.0）
    #[serde(default)]
    pub electricity_fee: f32,

    /// 主要房间路径（必填）
    #[validate(length(min = 1, max = 255, message = "房间路径长度必须在1-255字符之间"))]
    pub primary_roompath: String,
}

/// 更新阈值请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateThresholdRequest {
    /// 新的阈值
    #[validate(range(min = 0.0, message = "阈值不能为负数"))]
    pub threshold: f32,
}

/// 分页查询参数
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    /// 每页数量（默认20）
    #[serde(default = "default_limit")]
    pub limit: i64,

    /// 偏移量（默认0）
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

impl PaginationQuery {
    fn validate(&self) -> Result<()> {
        if !(MIN_PAGE_LIMIT..=MAX_PAGE_LIMIT).contains(&self.limit) {
            return Err(AppError::BadRequest(format!(
                "limit必须在{}到{}之间",
                MIN_PAGE_LIMIT, MAX_PAGE_LIMIT
            )));
        }

        if !(0..=MAX_PAGE_OFFSET).contains(&self.offset) {
            return Err(AppError::BadRequest(format!(
                "offset必须在0到{}之间",
                MAX_PAGE_OFFSET
            )));
        }

        Ok(())
    }
}

/// 房间响应
#[derive(Debug, Serialize)]
pub struct RoomResponse {
    pub id: Uuid,
    pub roomid: String,
    pub electricity_fee: f32,
    pub send_flag: bool,
    pub threshold: f32,
    pub room_name: String,

    // 同步相关字段
    pub primary_roompath: String,
    pub has_additional_paths: bool,
    pub is_active: bool,
    pub source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<String>,

    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::domain::models::Room> for RoomResponse {
    fn from(room: crate::domain::models::Room) -> Self {
        Self {
            id: room.id,
            roomid: roomid::to_string(room.roomid),
            electricity_fee: room.electricity_fee,
            send_flag: room.send_flag,
            threshold: room.threshold,
            room_name: room.room_name,
            primary_roompath: room.primary_roompath,
            has_additional_paths: room.has_additional_paths,
            is_active: room.is_active,
            source_type: room.source_type,
            external_id: room.external_id,
            last_synced_at: room.last_synced_at.map(|dt| dt.to_string()),
            created_at: room.created_at.to_string(),
            updated_at: room.updated_at.to_string(),
        }
    }
}

/// 创建房间
///
/// POST /api/rooms
pub async fn create_room(
    State(state): State<AppState>,
    Json(req): Json<CreateRoomRequest>,
) -> Result<(StatusCode, Json<RoomResponse>)> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::Internal(format!("验证失败: {}", e)))?;

    // 计算roompath的哈希值
    let primary_roompath_hash = calculate_roompath_hash(&req.primary_roompath);

    // 创建房间
    let new_room = NewRoom {
        roomid: req.roomid,
        electricity_fee: req.electricity_fee,
        threshold: req.threshold,
        room_name: req.room_name,
        primary_roompath: req.primary_roompath,
        primary_roompath_hash,
        has_additional_paths: false,
        is_active: true,
        source_type: "manual".to_string(),
        external_id: None,
        last_synced_at: None,
        last_recovered_at: None,
    };

    let room = RoomAccessUseCase::from_state(&state)
        .create_room(new_room)
        .await?;

    Ok((StatusCode::CREATED, Json(room.into())))
}

/// 获取房间详情
///
/// GET /api/rooms/{id}
///
/// 需要JWT认证，普通用户只能查询已绑定的房间
pub async fn get_room(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<RoomResponse>> {
    let actor = RoomActor::from_user_context(&user_ctx)?;
    let use_case = RoomAccessUseCase::from_state(&state);
    let room = use_case.get_room(&actor, id).await?;
    Ok(Json(room.into()))
}

/// 根据roomid查询房间
///
/// GET /api/rooms/by-roomid/{roomid}
///
/// 需要JWT认证，普通用户只能查询已绑定的房间
/// 注意：破坏性变更 - 现在返回单个Room（roomid现为唯一约束）
pub async fn get_rooms_by_roomid(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(roomid): Path<i64>,
) -> Result<Json<RoomResponse>> {
    let actor = RoomActor::from_user_context(&user_ctx)?;
    let use_case = RoomAccessUseCase::from_state(&state);
    let room = use_case.get_room_by_roomid(&actor, roomid).await?;
    Ok(Json(room.into()))
}

/// 更新房间阈值
///
/// PUT /api/rooms/{id}/threshold
///
/// 需要JWT认证，普通用户只能更新已绑定房间的阈值
pub async fn update_threshold(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateThresholdRequest>,
) -> Result<Json<RoomResponse>> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::Internal(format!("验证失败: {}", e)))?;

    let actor = RoomActor::from_user_context(&user_ctx)?;
    let use_case = RoomAccessUseCase::from_state(&state);
    let room = use_case.update_threshold(&actor, id, req.threshold).await?;
    Ok(Json(room.into()))
}

/// 手动重置send_flag
///
/// POST /api/rooms/{id}/reset-flag
pub async fn reset_send_flag(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RoomResponse>> {
    let room = RoomAccessUseCase::from_state(&state)
        .reset_send_flag(id)
        .await?;

    Ok(Json(room.into()))
}

/// 查询需要发送通知的房间
///
/// GET /api/rooms/flagged
///
/// 需要JWT认证，普通用户只能查询已绑定房间中需要通知的
pub async fn get_flagged_rooms(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
) -> Result<Json<Vec<RoomResponse>>> {
    let actor = RoomActor::from_user_context(&user_ctx)?;
    let use_case = RoomAccessUseCase::from_state(&state);
    let rooms = use_case.get_flagged_rooms(&actor).await?;
    Ok(Json(rooms.into_iter().map(Into::into).collect()))
}

/// 查询所有房间（分页）
///
/// GET /api/rooms
///
/// 需要JWT认证，普通用户只能查询已绑定的房间
pub async fn list_rooms(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Vec<RoomResponse>>> {
    pagination.validate()?;

    let actor = RoomActor::from_user_context(&user_ctx)?;
    let use_case = RoomAccessUseCase::from_state(&state);
    let rooms = use_case
        .list_rooms(&actor, pagination.limit, pagination.offset)
        .await?;
    Ok(Json(rooms.into_iter().map(Into::into).collect()))
}

/// 删除房间
///
/// DELETE /api/rooms/{id}
pub async fn delete_room(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let deleted = RoomAccessUseCase::from_state(&state)
        .delete_room(id)
        .await?;

    if deleted > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}
