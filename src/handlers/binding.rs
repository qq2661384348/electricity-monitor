//! 用户-房间绑定处理器
//!
//! 处理用户房间绑定相关的HTTP请求

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;
use validator::Validate;

use crate::config::AppConfig;
use crate::domain::models::{NewUserRoomBinding, UserRoomBinding};
use crate::errors::{AppError, Result};
use crate::infrastructure::repositories::{RoomRepository, UserRoomBindingRepository};
use crate::middleware::auth::UserContext;
use crate::state::AppState;

type HmacSha256 = Hmac<Sha256>;
const BINDING_PROOF_VERSION: &str = "v1";
const BINDING_PROOF_BYTES: usize = 6;

/// 创建绑定请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateBindingRequest {
    /// 房间ID
    pub roomid: i32,

    /// 是否启用通知（默认: false）
    #[serde(default)]
    pub notification_enabled: bool,

    /// 房间绑定证明码。
    ///
    /// 普通用户必须提供由管理员按房间生成的证明码；管理员创建自己的绑定时可省略。
    pub binding_proof: Option<String>,
}

/// 更新通知开关请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateNotificationRequest {
    /// 通知开关
    pub notification_enabled: bool,
}

/// 绑定响应
#[derive(Debug, Serialize)]
pub struct BindingResponse {
    pub id: String,
    pub user_id: String,
    pub roomid: i32,
    pub notification_enabled: bool,
    pub created_at: String,
    pub updated_at: String,

    // 完整的房间信息（联表查询时填充）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room: Option<crate::domain::models::Room>,
}

/// 管理员生成房间绑定证明码响应
#[derive(Debug, Serialize)]
pub struct BindingProofResponse {
    pub roomid: i32,
    pub binding_proof: String,
    pub proof_version: String,
}

impl From<UserRoomBinding> for BindingResponse {
    fn from(binding: UserRoomBinding) -> Self {
        Self {
            id: binding.id.to_string(),
            user_id: binding.user_id.to_string(),
            roomid: binding.roomid,
            notification_enabled: binding.notification_enabled,
            created_at: binding.created_at.to_string(),
            updated_at: binding.updated_at.to_string(),
            room: None,
        }
    }
}

impl BindingResponse {
    /// 从绑定和房间信息构造完整响应
    ///
    /// # 参数
    /// - `binding`: 用户房间绑定
    /// - `room`: 可选的房间信息（联表查询时提供）
    pub fn with_room_info(
        binding: UserRoomBinding,
        room: Option<&crate::domain::models::Room>,
    ) -> Self {
        let mut response = Self::from(binding);
        response.room = room.cloned();
        response
    }
}

/// 创建用户-房间绑定
///
/// POST /api/bindings
///
/// 需要认证
pub async fn create_binding(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Json(req): Json<CreateBindingRequest>,
) -> Result<(StatusCode, Json<BindingResponse>)> {
    let user_id_str = user_ctx
        .user_id
        .as_ref()
        .ok_or(AppError::Internal("用户ID缺失".to_string()))?;

    // 验证请求
    req.validate()
        .map_err(|e| AppError::Internal(format!("验证失败: {}", e)))?;

    tracing::info!(
        user_id = %user_id_str,
        roomid = req.roomid,
        "收到创建绑定请求"
    );

    // 解析user_id
    let user_id = Uuid::parse_str(user_id_str)
        .map_err(|_| AppError::Internal("无效的用户ID格式".to_string()))?;

    // 检查房间是否存在
    let room_repo = RoomRepository::new(state.db_pool.clone());
    if room_repo.find_by_roomid(req.roomid).await?.is_none() {
        return Err(AppError::NotFound);
    }

    if !user_ctx.is_admin {
        validate_binding_proof(req.roomid, req.binding_proof.as_deref())?;
    }

    // 检查是否已经绑定
    let binding_repo = UserRoomBindingRepository::new(state.db_pool.clone());
    if let Some(existing) = binding_repo
        .find_by_user_and_room(user_id, req.roomid)
        .await?
    {
        tracing::warn!(
            user_id = %user_id,
            roomid = req.roomid,
            "用户已绑定该房间"
        );
        return Ok((StatusCode::OK, Json(existing.into())));
    }

    // 创建绑定
    let new_binding = NewUserRoomBinding {
        user_id,
        roomid: req.roomid,
        notification_enabled: req.notification_enabled,
    };

    let binding = binding_repo.create(new_binding).await?;
    state.cache_manager.invalidate_binding(req.roomid).await?;

    tracing::info!(
        user_id = %user_id,
        roomid = req.roomid,
        binding_id = %binding.id,
        "创建绑定成功"
    );

    Ok((StatusCode::CREATED, Json(binding.into())))
}

/// 管理员生成房间绑定证明码
///
/// GET /api/bindings/proof/{roomid}
///
/// 需要管理员权限。证明码不落库，基于服务端签名密钥和 roomid 生成；
/// 这样可以在不新增房间密钥表的前提下阻断普通用户只凭 roomid 自助授权。
pub async fn get_binding_proof(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(roomid): Path<i32>,
) -> Result<Json<BindingProofResponse>> {
    if !user_ctx.is_admin {
        return Err(AppError::Forbidden);
    }

    let room_repo = RoomRepository::new(state.db_pool.clone());
    if room_repo.find_by_roomid(roomid).await?.is_none() {
        return Err(AppError::NotFound);
    }

    Ok(Json(BindingProofResponse {
        roomid,
        binding_proof: room_binding_proof(roomid)?,
        proof_version: BINDING_PROOF_VERSION.to_string(),
    }))
}

fn validate_binding_proof(roomid: i32, proof: Option<&str>) -> Result<()> {
    let provided = proof
        .map(normalize_binding_proof)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::Forbidden)?;
    let expected = room_binding_proof(roomid)?;

    if constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn room_binding_proof(roomid: i32) -> Result<String> {
    let secret = AppConfig::global().jwt.secret.as_bytes();
    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|_| AppError::Internal("绑定证明码签名密钥无效".to_string()))?;
    mac.update(format!("room-binding:{BINDING_PROOF_VERSION}:{roomid}").as_bytes());
    let digest = mac.finalize().into_bytes();

    Ok(digest[..BINDING_PROOF_BYTES]
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect())
}

fn normalize_binding_proof(proof: &str) -> String {
    proof
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace() && *ch != '-')
        .flat_map(|ch| ch.to_uppercase())
        .collect()
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut diff = left.len() ^ right.len();
    let len = left.len().max(right.len());

    for index in 0..len {
        let left_byte = left.get(index).copied().unwrap_or(0);
        let right_byte = right.get(index).copied().unwrap_or(0);
        diff |= usize::from(left_byte ^ right_byte);
    }

    diff == 0
}

/// 查询用户的所有绑定（包含房间信息）
///
/// GET /api/bindings
///
/// 需要认证
///
/// # 性能优化
/// 使用批量查询避免N+1问题：
/// 1. 查询所有绑定
/// 2. 批量查询相关房间
/// 3. 内存中组装数据
pub async fn list_bindings(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
) -> Result<Json<Vec<BindingResponse>>> {
    let user_id_str = user_ctx
        .user_id
        .as_ref()
        .ok_or(AppError::Internal("用户ID缺失".to_string()))?;

    // 解析user_id
    let user_id = Uuid::parse_str(user_id_str)
        .map_err(|_| AppError::Internal("无效的用户ID格式".to_string()))?;

    // 1. 查询所有绑定
    let binding_repo = UserRoomBindingRepository::new(state.db_pool.clone());
    let bindings = binding_repo.find_by_user_id(user_id).await?;

    // 如果没有绑定，直接返回空列表
    if bindings.is_empty() {
        return Ok(Json(Vec::new()));
    }

    // 2. 收集所有roomid
    let roomids: Vec<i32> = bindings.iter().map(|b| b.roomid).collect();

    // 3. 批量查询房间信息
    let room_repo = RoomRepository::new(state.db_pool.clone());
    let rooms = room_repo.find_by_roomids(&roomids).await?;

    // 4. 构建roomid -> Room 的映射，方便快速查找
    use std::collections::HashMap;
    let room_map: HashMap<i32, &crate::domain::models::Room> =
        rooms.iter().map(|r| (r.roomid, r)).collect();

    // 5. 组装响应，填充房间信息
    let responses: Vec<BindingResponse> = bindings
        .into_iter()
        .map(|binding| {
            let room = room_map.get(&binding.roomid).copied();
            BindingResponse::with_room_info(binding, room)
        })
        .collect();

    tracing::debug!(
        user_id = %user_id_str,
        binding_count = responses.len(),
        "查询绑定列表成功（含房间信息）"
    );

    Ok(Json(responses))
}

/// 获取绑定详情
///
/// GET /api/bindings/{id}
///
/// 需要认证，只能查看自己的绑定（管理员除外）
pub async fn get_binding(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<BindingResponse>> {
    let binding_repo = UserRoomBindingRepository::new(state.db_pool.clone());

    let binding = binding_repo
        .find_by_id(id)
        .await?
        .ok_or(AppError::NotFound)?;

    // 管理员可以查看所有绑定
    if !user_ctx.is_admin {
        // 普通用户只能查看自己的绑定
        let user_id_str = user_ctx
            .user_id
            .as_ref()
            .ok_or(AppError::Internal("用户ID缺失".to_string()))?;

        if binding.user_id.to_string() != *user_id_str {
            tracing::warn!(
                user_id = %user_id_str,
                binding_id = %id,
                "用户尝试访问他人的绑定"
            );
            return Err(AppError::Unauthorized("无权访问该绑定".to_string()));
        }
    }

    Ok(Json(binding.into()))
}

/// 更新通知开关
///
/// PUT /api/bindings/{id}/notification
///
/// 需要认证，只能更新自己的绑定（管理员除外）
pub async fn update_notification(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateNotificationRequest>,
) -> Result<Json<BindingResponse>> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::Internal(format!("验证失败: {}", e)))?;

    let binding_repo = UserRoomBindingRepository::new(state.db_pool.clone());

    // 查询绑定
    let binding = binding_repo
        .find_by_id(id)
        .await?
        .ok_or(AppError::NotFound)?;

    // 管理员可以更新所有绑定
    if !user_ctx.is_admin {
        // 普通用户只能更新自己的绑定
        let user_id_str = user_ctx
            .user_id
            .as_ref()
            .ok_or(AppError::Internal("用户ID缺失".to_string()))?;

        if binding.user_id.to_string() != *user_id_str {
            tracing::warn!(
                user_id = %user_id_str,
                binding_id = %id,
                "用户尝试更新他人的绑定"
            );
            return Err(AppError::Unauthorized("无权修改该绑定".to_string()));
        }
    }

    // 更新通知开关
    let updated_binding = binding_repo
        .update_notification_enabled(id, req.notification_enabled)
        .await?;
    state
        .cache_manager
        .invalidate_binding(binding.roomid)
        .await?;

    let user_info = if user_ctx.is_admin {
        "admin".to_string()
    } else {
        user_ctx
            .user_id
            .as_ref()
            .unwrap_or(&"unknown".to_string())
            .clone()
    };

    tracing::info!(
        user_id = %user_info,
        binding_id = %id,
        notification_enabled = req.notification_enabled,
        "更新通知开关成功"
    );

    Ok(Json(updated_binding.into()))
}

/// 删除绑定
///
/// DELETE /api/bindings/{id}
///
/// 需要认证，只能删除自己的绑定（管理员除外）
pub async fn delete_binding(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let binding_repo = UserRoomBindingRepository::new(state.db_pool.clone());

    // 查询绑定
    let binding = binding_repo
        .find_by_id(id)
        .await?
        .ok_or(AppError::NotFound)?;

    // 管理员可以删除所有绑定
    if !user_ctx.is_admin {
        // 普通用户只能删除自己的绑定
        let user_id_str = user_ctx
            .user_id
            .as_ref()
            .ok_or(AppError::Internal("用户ID缺失".to_string()))?;

        if binding.user_id.to_string() != *user_id_str {
            tracing::warn!(
                user_id = %user_id_str,
                binding_id = %id,
                "用户尝试删除他人的绑定"
            );
            return Err(AppError::Unauthorized("无权删除该绑定".to_string()));
        }
    }

    // 删除绑定
    let deleted = binding_repo.delete(id).await?;
    if deleted > 0 {
        state
            .cache_manager
            .invalidate_binding(binding.roomid)
            .await?;
    }

    if deleted > 0 {
        let user_info = if user_ctx.is_admin {
            "admin".to_string()
        } else {
            user_ctx
                .user_id
                .as_ref()
                .unwrap_or(&"unknown".to_string())
                .clone()
        };

        tracing::info!(
            user_id = %user_info,
            binding_id = %id,
            "删除绑定成功"
        );
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}
