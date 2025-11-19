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

use crate::domain::models::{NewRoom, UpdateThreshold};
use crate::errors::{AppError, Result};
use crate::infrastructure::repositories::{RoomRepository, UserRoomBindingRepository};
use crate::middleware::auth::UserContext;
use crate::state::AppState;
use crate::utils::hash::calculate_roompath_hash;

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

/// 房间响应
#[derive(Debug, Serialize)]
pub struct RoomResponse {
    pub id: Uuid,
    pub roomid: i32,
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
            roomid: room.roomid,
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

    // 创建Repository
    let repository = RoomRepository::new(state.db_pool.clone());

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
    };

    let room = repository.create(new_room).await?;

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
    let repository = RoomRepository::new(state.db_pool.clone());

    let room = repository
        .find_by_id(id)
        .await?
        .ok_or(AppError::NotFound)?;

    // 管理员可以查询所有房间
    if user_ctx.is_admin {
        return Ok(Json(room.into()));
    }

    // 普通用户只能查询已绑定的房间
    if let Some(user_id_str) = &user_ctx.user_id {
        let user_id = Uuid::parse_str(user_id_str)
            .map_err(|_| AppError::Internal("无效的用户ID格式".to_string()))?;
        
        let binding_repo = UserRoomBindingRepository::new(state.db_pool.clone());
        let binding = binding_repo
            .find_by_user_and_room(user_id, room.roomid)
            .await?;
        
        if binding.is_none() {
            tracing::warn!(
                user_id = %user_id_str,
                roomid = room.roomid,
                "普通用户尝试访问未绑定的房间"
            );
            return Err(AppError::Forbidden);
        }
        
        return Ok(Json(room.into()));
    }

    // 未认证
    Err(AppError::Unauthorized("未认证".to_string()))
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
    Path(roomid): Path<i32>,
) -> Result<Json<RoomResponse>> {
    let repository = RoomRepository::new(state.db_pool.clone());

    let room = repository
        .find_by_roomid(roomid)
        .await?
        .ok_or(AppError::NotFound)?;

    // 管理员可以查询所有房间
    if user_ctx.is_admin {
        return Ok(Json(room.into()));
    }

    // 普通用户只能查询已绑定的房间
    if let Some(user_id_str) = &user_ctx.user_id {
        let user_id = Uuid::parse_str(user_id_str)
            .map_err(|_| AppError::Internal("无效的用户ID格式".to_string()))?;
        
        let binding_repo = UserRoomBindingRepository::new(state.db_pool.clone());
        let binding = binding_repo
            .find_by_user_and_room(user_id, roomid)
            .await?;
        
        if binding.is_none() {
            tracing::warn!(
                user_id = %user_id_str,
                roomid = roomid,
                "普通用户尝试访问未绑定的房间"
            );
            return Err(AppError::Forbidden);
        }
        
        return Ok(Json(room.into()));
    }

    // 未认证
    Err(AppError::Unauthorized("未认证".to_string()))
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

    let repository = RoomRepository::new(state.db_pool.clone());

    // 先查询房间是否存在
    let room = repository
        .find_by_id(id)
        .await?
        .ok_or(AppError::NotFound)?;

    // 管理员可以更新所有房间
    if !user_ctx.is_admin {
        // 普通用户只能更新已绑定房间的阈值
        if let Some(user_id_str) = &user_ctx.user_id {
            let user_id = Uuid::parse_str(user_id_str)
                .map_err(|_| AppError::Internal("无效的用户ID格式".to_string()))?;
            
            let binding_repo = UserRoomBindingRepository::new(state.db_pool.clone());
            let binding = binding_repo
                .find_by_user_and_room(user_id, room.roomid)
                .await?;
            
            if binding.is_none() {
                tracing::warn!(
                    user_id = %user_id_str,
                    roomid = room.roomid,
                    "普通用户尝试更新未绑定房间的阈值"
                );
                return Err(AppError::Forbidden);
            }
        } else {
            return Err(AppError::Unauthorized("未认证".to_string()));
        }
    }

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
/// 
/// 需要JWT认证，普通用户只能查询已绑定房间中需要通知的
/// 
/// # 性能优化
/// 使用内存缓存(state.flagged_rooms_cache)避免全量数据库查询
/// 使用内存过滤避免N+1查询
pub async fn get_flagged_rooms(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
) -> Result<Json<Vec<RoomResponse>>> {
    // 1. 从缓存获取全量 Flagged 房间
    // 使用异步读取锁
    let rooms_snapshot = {
        let cache = state.flagged_rooms_cache.read().await;
        cache.clone()
    };

    // 如果缓存为空，可能是服务刚启动，尝试直接查库作为降级方案
    if rooms_snapshot.is_empty() {
        tracing::warn!("Flagged Rooms缓存为空，降级为数据库查询");
        let repository = RoomRepository::new(state.db_pool.clone());
        let rooms = repository.find_rooms_with_send_flag_true().await?;
        
        // 管理员返回所有
        if user_ctx.is_admin {
            let responses: Vec<RoomResponse> = rooms.into_iter().map(Into::into).collect();
            return Ok(Json(responses));
        }

        // 普通用户过滤
        if let Some(user_id_str) = &user_ctx.user_id {
            let user_id = Uuid::parse_str(user_id_str)
                .map_err(|_| AppError::Internal("无效的用户ID格式".to_string()))?;
            
            let binding_repo = UserRoomBindingRepository::new(state.db_pool.clone());
            let bindings = binding_repo.find_by_user_id(user_id).await?;
            let bound_roomids: std::collections::HashSet<i32> = bindings.iter().map(|b| b.roomid).collect();
            
            let filtered: Vec<RoomResponse> = rooms.into_iter()
                .filter(|r| bound_roomids.contains(&r.roomid))
                .map(Into::into)
                .collect();
            return Ok(Json(filtered));
        }
        return Err(AppError::Unauthorized("未认证".to_string()));
    }

    // 2. 管理员可以直接返回所有数据
    if user_ctx.is_admin {
        let responses: Vec<RoomResponse> = rooms_snapshot.into_iter().map(Into::into).collect();
        return Ok(Json(responses));
    }

    // 3. 普通用户：内存过滤
    if let Some(user_id_str) = &user_ctx.user_id {
        let user_id = Uuid::parse_str(user_id_str)
            .map_err(|_| AppError::Internal("无效的用户ID格式".to_string()))?;
        
        // 获取用户绑定列表 (1次数据库查询，索引扫描)
        let binding_repo = UserRoomBindingRepository::new(state.db_pool.clone());
        let bindings = binding_repo.find_by_user_id(user_id).await?;
        
        // 构建HashSet用于O(1)查找
        let bound_roomids: std::collections::HashSet<i32> = bindings.iter().map(|b| b.roomid).collect();
        
        // 内存过滤
        let responses: Vec<RoomResponse> = rooms_snapshot.into_iter()
            .filter(|r| bound_roomids.contains(&r.roomid))
            .map(Into::into)
            .collect();
            
        return Ok(Json(responses));
    }

    // 未认证
    Err(AppError::Unauthorized("未认证".to_string()))
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
    let repository = RoomRepository::new(state.db_pool.clone());

    // 管理员可以查询所有房间
    if user_ctx.is_admin {
        let rooms = repository
            .find_all(pagination.limit, pagination.offset)
            .await?;
        let responses: Vec<RoomResponse> = rooms.into_iter().map(Into::into).collect();
        return Ok(Json(responses));
    }

    // 普通用户只能查询已绑定的房间
    if let Some(user_id_str) = &user_ctx.user_id {
        let user_id = Uuid::parse_str(user_id_str)
            .map_err(|_| AppError::Internal("无效的用户ID格式".to_string()))?;
        
        let binding_repo = UserRoomBindingRepository::new(state.db_pool.clone());
        let bindings = binding_repo.find_by_user_id(user_id).await?;
        
        // 获取用户已绑定的房间ID列表
        let bound_roomids: Vec<i32> = bindings.iter().map(|b| b.roomid).collect();
        
        // 使用find_by_roomids_paged代替先查全量再过滤
        // 这解决了分页逻辑错误（先分页后过滤导致空页）
        let filtered_rooms = repository
            .find_by_roomids_paged(&bound_roomids, pagination.limit, pagination.offset)
            .await?;
        
        let responses: Vec<RoomResponse> = filtered_rooms.into_iter().map(Into::into).collect();
        return Ok(Json(responses));
    }

    // 未认证
    Err(AppError::Unauthorized("未认证".to_string()))
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
