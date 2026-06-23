//! 房间同步API处理器

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::services::SyncStats;
use crate::errors::Result;
use crate::modules::room_sync::application::RoomSyncUseCase;
use crate::state::AppState;

/// 同步触发响应
#[derive(Debug, Serialize)]
pub struct SyncTriggerResponse {
    /// 任务ID
    pub job_id: Uuid,

    /// 任务状态
    pub status: String,

    /// 消息
    pub message: String,
}

/// 手动触发房间同步
///
/// POST /api/rooms/sync
///
/// 触发异步同步任务，立即返回任务ID
pub async fn trigger_sync(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<SyncTriggerResponse>)> {
    tracing::info!("收到手动同步触发请求");
    let job_id = RoomSyncUseCase::from_state(&state).trigger_sync().await?;

    let response = SyncTriggerResponse {
        job_id,
        status: "pending".to_string(),
        message: "同步任务已启动".to_string(),
    };

    Ok((StatusCode::ACCEPTED, Json(response)))
}

/// 查询房间的所有路径（聚合根）
///
/// GET /api/rooms/{roomid}/paths
pub async fn get_room_paths(
    State(state): State<AppState>,
    Path(roomid): Path<i64>,
) -> Result<Json<RoomPathsResponse>> {
    let aggregate = RoomSyncUseCase::from_state(&state)
        .get_room_paths(roomid)
        .await?;

    let total_paths = aggregate.all_roompaths().len();
    let roomid = aggregate.room.roomid;
    let primary_roompath = aggregate.room.primary_roompath.clone();
    let has_additional_paths = aggregate.room.has_additional_paths;

    let additional_paths: Vec<PathInfo> = aggregate
        .additional_paths
        .into_iter()
        .map(|p| PathInfo {
            id: p.id,
            roompath: p.roompath,
            room_name: p.room_name,
            created_at: p.created_at.to_string(),
        })
        .collect();

    let response = RoomPathsResponse {
        roomid,
        primary_roompath,
        has_additional_paths,
        additional_paths,
        total_paths,
    };

    Ok(Json(response))
}

/// 房间路径响应
#[derive(Debug, Serialize)]
pub struct RoomPathsResponse {
    pub roomid: i64,
    pub primary_roompath: String,
    pub has_additional_paths: bool,
    pub additional_paths: Vec<PathInfo>,
    pub total_paths: usize,
}

/// 路径信息
#[derive(Debug, Serialize)]
pub struct PathInfo {
    pub id: Uuid,
    pub roompath: String,
    pub room_name: String,
    pub created_at: String,
}

/// 同步任务状态响应
#[derive(Debug, Serialize)]
pub struct SyncStatusResponse {
    pub job_id: Uuid,
    pub status: String,
    pub message: String,
}

/// 查询同步任务状态
///
/// GET /api/rooms/sync/status/{job_id}
///
/// 从数据库查询真实的同步任务状态
pub async fn get_sync_status(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<SyncStatusResponse>> {
    match RoomSyncUseCase::from_state(&state)
        .get_sync_status(job_id)
        .await?
    {
        Some(log) => {
            let message = if let Some(err) = log.error_message.clone() {
                format!("任务失败: {}", err)
            } else if log.completed_at.is_some() {
                "任务已完成".to_string()
            } else {
                format!(
                    "任务运行中（开始于: {}）",
                    log.started_at.format("%Y-%m-%d %H:%M:%S")
                )
            };

            Ok(Json(SyncStatusResponse {
                job_id: log.id,
                status: log.status,
                message,
            }))
        }
        None => Ok(Json(SyncStatusResponse {
            job_id,
            status: "pending".to_string(),
            message: "任务队列中，尚未开始执行".to_string(),
        })),
    }
}

/// 查询同步历史
///
/// GET /api/rooms/sync/history
///
/// 返回最近的同步历史记录（默认10条）
pub async fn get_sync_history(State(state): State<AppState>) -> Result<Json<Vec<SyncHistoryItem>>> {
    let logs = RoomSyncUseCase::from_state(&state)
        .get_sync_history(10)
        .await?;

    let items: Vec<SyncHistoryItem> = logs
        .into_iter()
        .map(|log| {
            // 解析stats JSON
            let stats = log
                .stats
                .and_then(|v| serde_json::from_value::<SyncStats>(v).ok());

            SyncHistoryItem {
                id: log.id,
                sync_type: log.sync_type,
                started_at: log.started_at.to_string(),
                completed_at: log.completed_at.map(|dt| dt.to_string()),
                status: log.status,
                stats,
                error_message: log.error_message,
            }
        })
        .collect();

    Ok(Json(items))
}

/// 同步历史项
#[derive(Debug, Serialize)]
pub struct SyncHistoryItem {
    pub id: Uuid,
    pub sync_type: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub status: String,
    pub stats: Option<SyncStats>,
    pub error_message: Option<String>,
}
