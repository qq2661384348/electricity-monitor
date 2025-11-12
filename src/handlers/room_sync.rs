//! 房间同步API处理器

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::services::{RoomSyncService, SyncStats};
use crate::errors::{AppError, Result};
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
    
    // 生成任务ID
    let job_id = Uuid::new_v4();
    
    // 克隆必要的状态
    let db_pool = state.db_pool.clone();
    let electricity_fetcher = state.electricity_fetcher_service.clone();
    let config = crate::config::AppConfig::global();
    
    // 启动异步任务
    tokio::spawn(async move {
        tracing::info!("开始异步同步任务: job_id={}", job_id);
        
        // 创建仓储
        let repository = std::sync::Arc::new(
            crate::infrastructure::repositories::RoomRepository::new(db_pool)
        );
        
        // 创建同步日志记录
        let log_id = job_id;  // 使用job_id作为日志ID
        let new_log = crate::domain::models::NewRoomSyncLog {
            id: Some(log_id),  // 显式设置ID
            sync_type: "manual".to_string(),
            started_at: chrono::Utc::now().naive_utc(),
            status: "running".to_string(),
        };
        
        // 记录同步开始
        if let Err(e) = repository.create_sync_log(new_log).await {
            tracing::error!("创建同步日志失败: job_id={}, error={}", job_id, e);
            return;
        }
        
        // 创建爬虫客户端和获取器
        let client = match crate::domain::services::RoomClient::new(&config.room_sync.crawler) {
            Ok(c) => std::sync::Arc::new(c),
            Err(e) => {
                tracing::error!("创建爬虫客户端失败: job_id={}, error={}", job_id, e);
                
                // 更新日志为失败状态
                let update_log = crate::domain::models::UpdateRoomSyncLog {
                    completed_at: Some(chrono::Utc::now().naive_utc()),
                    status: Some("failed".to_string()),
                    stats: None,
                    error_message: Some(format!("创建爬虫客户端失败: {}", e)),
                };
                let _ = repository.update_sync_log(log_id, update_log).await;
                return;
            }
        };
        
        let fetcher = std::sync::Arc::new(crate::domain::services::RoomFetcher::new(client));
        
        // 创建同步服务
        let sync_service = RoomSyncService::new(
            repository.clone(),
            fetcher,
            config.room_sync.default_threshold,
        );
        
        // 执行同步
        match sync_service.sync().await {
            Ok(stats) => {
                tracing::info!(
                    "同步任务完成: job_id={}, 新增={}, 更新={}, 失败={}",
                    job_id,
                    stats.new,
                    stats.updated,
                    stats.failed
                );
                
                // 更新日志为成功状态
                let stats_json = serde_json::to_value(&stats).ok();
                let update_log = crate::domain::models::UpdateRoomSyncLog {
                    completed_at: Some(chrono::Utc::now().naive_utc()),
                    status: Some("completed".to_string()),
                    stats: stats_json,
                    error_message: None,
                };
                
                if let Err(e) = repository.update_sync_log(log_id, update_log).await {
                    tracing::error!("更新同步日志失败: job_id={}, error={}", job_id, e);
                }
                
                // ✅ 同步成功后刷新ElectricityFetcher缓存
                if let Some(fetcher_service) = electricity_fetcher.as_ref() {
                    tracing::info!("刷新电费获取服务缓存: job_id={}", job_id);
                    match fetcher_service.refresh_cache().await {
                        Ok(_) => {
                            let cache_size = fetcher_service.cache_size().await;
                            tracing::info!(
                                "缓存刷新成功: job_id={}, cache_size={}",
                                job_id,
                                cache_size
                            );
                        }
                        Err(e) => {
                            tracing::error!(
                                "缓存刷新失败: job_id={}, error={}",
                                job_id,
                                e
                            );
                        }
                    }
                } else {
                    tracing::debug!("电费获取服务未启用，跳过缓存刷新");
                }
            }
            Err(e) => {
                tracing::error!("同步任务失败: job_id={}, error={}", job_id, e);
                
                // 更新日志为失败状态
                let update_log = crate::domain::models::UpdateRoomSyncLog {
                    completed_at: Some(chrono::Utc::now().naive_utc()),
                    status: Some("failed".to_string()),
                    stats: None,
                    error_message: Some(e.to_string()),
                };
                
                if let Err(e) = repository.update_sync_log(log_id, update_log).await {
                    tracing::error!("更新同步日志失败: job_id={}, error={}", job_id, e);
                }
            }
        }
    });
    
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
    Path(roomid): Path<i32>,
) -> Result<Json<RoomPathsResponse>> {
    let repository = crate::infrastructure::repositories::RoomRepository::new(state.db_pool.clone());
    
    // 查询聚合根
    let aggregate = repository.find_room_with_all_paths(roomid)
        .await?
        .ok_or(AppError::NotFound)?;
    
    let total_paths = aggregate.all_roompaths().len();
    let roomid = aggregate.room.roomid;
    let primary_roompath = aggregate.room.primary_roompath.clone();
    let has_additional_paths = aggregate.room.has_additional_paths;
    
    let additional_paths: Vec<PathInfo> = aggregate.additional_paths
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
    pub roomid: i32,
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
    use crate::infrastructure::database::schema::room_sync_log;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    
    // 从数据库查询任务记录
    let mut conn = state.db_pool.get().await.map_err(|e| {
        AppError::Internal(format!("Failed to get database connection: {}", e))
    })?;
    
    let log = room_sync_log::table
        .find(job_id)
        .select((
            room_sync_log::id,
            room_sync_log::status,
            room_sync_log::started_at,
            room_sync_log::completed_at,
            room_sync_log::error_message,
        ))
        .first::<(Uuid, String, chrono::NaiveDateTime, Option<chrono::NaiveDateTime>, Option<String>)>(&mut conn)
        .await
        .optional()
        .map_err(AppError::Database)?;
    
    match log {
        Some((id, status, started_at, completed_at, error_msg)) => {
            let message = if let Some(err) = error_msg {
                format!("任务失败: {}", err)
            } else if completed_at.is_some() {
                "任务已完成".to_string()
            } else {
                format!("任务运行中（开始于: {}）", started_at.format("%Y-%m-%d %H:%M:%S"))
            };
            
            Ok(Json(SyncStatusResponse {
                job_id: id,
                status,
                message,
            }))
        },
        None => {
            // 任务不存在，可能尚未记录到数据库
            Ok(Json(SyncStatusResponse {
                job_id,
                status: "pending".to_string(),
                message: "任务队列中，尚未开始执行".to_string(),
            }))
        }
    }
}

/// 查询同步历史
/// 
/// GET /api/rooms/sync/history
/// 
/// 返回最近的同步历史记录（默认10条）
pub async fn get_sync_history(
    State(state): State<AppState>,
) -> Result<Json<Vec<SyncHistoryItem>>> {
    let repository = crate::infrastructure::repositories::RoomRepository::new(state.db_pool.clone());
    
    // 查询最近10条记录
    let logs = repository.get_sync_history(10).await?;
    
    let items: Vec<SyncHistoryItem> = logs
        .into_iter()
        .map(|log| {
            // 解析stats JSON
            let stats = log.stats.and_then(|v| {
                serde_json::from_value::<SyncStats>(v).ok()
            });
            
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
