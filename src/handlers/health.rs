//! 健康检查处理器

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::state::AppState;

/// 健康检查端点
///
/// GET /api/health
pub async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "message": "Service is healthy"
        }))
    )
}

/// 健康检查端点（带数据库检查）
///
/// GET /api/health/db
pub async fn health_check_db(State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: 实际检查数据库连接
    // let _conn = state.db_pool.get().await.map_err(|_| {
    //     (StatusCode::SERVICE_UNAVAILABLE, "Database unavailable")
    // })?;

    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "database": "connected",
            "message": "Service and database are healthy"
        }))
    )
}
