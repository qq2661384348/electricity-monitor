//! 电费获取Handler

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::state::AppState;
use crate::errors::Result;

/// 手动触发电费获取请求
#[derive(Debug, Deserialize)]
pub struct TriggerFetchRequest {
    /// 是否同时触发历史记录任务
    #[serde(default)]
    pub with_history: bool,
}

/// 手动触发响应
#[derive(Debug, Serialize)]
pub struct TriggerFetchResponse {
    pub message: String,
    pub success_count: usize,
    pub failure_count: usize,
    pub updated_count: usize,
    pub duration_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_inserted: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_deleted: Option<usize>,
}

/// 手动触发电费获取
///
/// POST /api/electricity/fetch
pub async fn trigger_fetch(
    State(state): State<AppState>,
    Json(req): Json<TriggerFetchRequest>,
) -> Result<impl IntoResponse> {
    tracing::info!(with_history = req.with_history, "手动触发电费获取");

    // 如果ElectricityFetcherService已初始化，使用它
    if let Some(fetcher_service) = &state.electricity_fetcher_service {
        // 执行电费获取
        let stats = fetcher_service.run_fetch_task().await?;

        // 如果需要，执行历史记录任务
        let (history_inserted, history_deleted) = if req.with_history {
            let (inserted, deleted) = fetcher_service.run_history_task().await?;
            (Some(inserted), Some(deleted))
        } else {
            (None, None)
        };

        Ok((
            StatusCode::OK,
            Json(TriggerFetchResponse {
                message: "电费获取完成".to_string(),
                success_count: stats.success_count,
                failure_count: stats.failure_count,
                updated_count: stats.updated_count,
                duration_ms: stats.duration_ms,
                history_inserted,
                history_deleted,
            }),
        ))
    } else {
        Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(TriggerFetchResponse {
                message: "电费获取服务未启用".to_string(),
                success_count: 0,
                failure_count: 0,
                updated_count: 0,
                duration_ms: 0,
                history_inserted: None,
                history_deleted: None,
            }),
        ))
    }
}

/// 刷新RoomId缓存
///
/// POST /api/electricity/refresh-cache
pub async fn refresh_cache(State(state): State<AppState>) -> Result<impl IntoResponse> {
    tracing::info!("手动刷新RoomId缓存");

    if let Some(fetcher_service) = &state.electricity_fetcher_service {
        fetcher_service.refresh_cache().await?;
        let size = fetcher_service.cache_size().await;

        Ok((
            StatusCode::OK,
            Json(json!({
                "message": "缓存刷新完成",
                "cache_size": size
            })),
        ))
    } else {
        Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "message": "电费获取服务未启用",
                "cache_size": 0
            })),
        ))
    }
}

/// 获取服务状态
///
/// GET /api/electricity/status
pub async fn get_status(State(state): State<AppState>) -> Result<impl IntoResponse> {
    if let Some(fetcher_service) = &state.electricity_fetcher_service {
        let cache_size = fetcher_service.cache_size().await;

        Ok((
            StatusCode::OK,
            Json(json!({
                "enabled": true,
                "cache_size": cache_size,
                "message": "电费获取服务正常运行"
            })),
        ))
    } else {
        Ok((
            StatusCode::OK,
            Json(json!({
                "enabled": false,
                "cache_size": 0,
                "message": "电费获取服务未启用"
            })),
        ))
    }
}
