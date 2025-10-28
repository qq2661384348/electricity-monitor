//! Room处理器
//! 
//! 处理房间相关的HTTP请求

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::models::{NewRoom, UpdateThreshold};
use crate::errors::{AppError, Result};
use crate::infrastructure::repositories::RoomRepository;
use crate::state::AppState;

/// 创建房间请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateRoomRequest {
    /// 房间ID
    pub roomid: i32,
    
    /// 电费阈值
    #[validate(range(min = 0.0, message = "阈值不能为负数"))]
    pub threshold: f32,
    
    /// 房间名称
    #[validate(length(min = 1, max = 64, message = "房间名称长度必须在1-64字符之间"))]
    pub room_name: String,
    
    /// 初始电费（可选，默认0.0）
    #[serde(default)]
    pub electricity_fee: f32,
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

/// 房间响应
#[derive(Debug, Serialize)]
pub struct RoomResponse {
    pub id: Uuid,
    pub roomid: i32,
    pub electricity_fee: f32,
    pub send_flag: bool,
    pub threshold: f32,
    pub room_name: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::domain::models::Room> for RoomResponse {
    fn from(room: crate::domain::models::Room) -> Self {
        Self {
            id: room.id,
            roomid: room.roomid,
            electricity_fee: room.electricity_fee,
            send_flag: room.send_flag,
            threshold: room.threshold,
            room_name: room.room_name,
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

    // 创建Repository
    let repository = RoomRepository::new(state.db_pool.clone());

    // 创建房间
    let new_room = NewRoom {
        roomid: req.roomid,
        electricity_fee: req.electricity_fee,
        threshold: req.threshold,
        room_name: req.room_name,
    };

    let room = repository.create(new_room).await?;

    Ok((StatusCode::CREATED, Json(room.into())))
}

/// 获取房间详情
/// 
/// GET /api/rooms/{id}
pub async fn get_room(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RoomResponse>> {
    let repository = RoomRepository::new(state.db_pool.clone());

    let room = repository
        .find_by_id(id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(room.into()))
}

/// 根据roomid查询房间
/// 
/// GET /api/rooms/by-roomid/{roomid}
pub async fn get_rooms_by_roomid(
    State(state): State<AppState>,
    Path(roomid): Path<i32>,
) -> Result<Json<Vec<RoomResponse>>> {
    let repository = RoomRepository::new(state.db_pool.clone());

    let rooms = repository.find_by_roomid(roomid).await?;

    let responses: Vec<RoomResponse> = rooms.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

/// 更新房间阈值
/// 
/// PUT /api/rooms/{id}/threshold
pub async fn update_threshold(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateThresholdRequest>,
) -> Result<Json<RoomResponse>> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::Internal(format!("验证失败: {}", e)))?;

    let repository = RoomRepository::new(state.db_pool.clone());

    let update = UpdateThreshold {
        threshold: req.threshold,
    };

    let room = repository.update_threshold(id, update).await?;

    Ok(Json(room.into()))
}

/// 手动重置send_flag
/// 
/// POST /api/rooms/{id}/reset-flag
pub async fn reset_send_flag(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RoomResponse>> {
    let repository = RoomRepository::new(state.db_pool.clone());

    let room = repository.reset_send_flag(id).await?;

    Ok(Json(room.into()))
}

/// 查询需要发送通知的房间
/// 
/// GET /api/rooms/flagged
pub async fn get_flagged_rooms(
    State(state): State<AppState>,
) -> Result<Json<Vec<RoomResponse>>> {
    let repository = RoomRepository::new(state.db_pool.clone());

    let rooms = repository.find_rooms_with_send_flag_true().await?;

    let responses: Vec<RoomResponse> = rooms.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

/// 查询所有房间（分页）
/// 
/// GET /api/rooms
pub async fn list_rooms(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Vec<RoomResponse>>> {
    let repository = RoomRepository::new(state.db_pool.clone());

    let rooms = repository
        .find_all(pagination.limit, pagination.offset)
        .await?;

    let responses: Vec<RoomResponse> = rooms.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

/// 删除房间
/// 
/// DELETE /api/rooms/{id}
pub async fn delete_room(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let repository = RoomRepository::new(state.db_pool.clone());

    let deleted = repository.delete(id).await?;

    if deleted > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}
