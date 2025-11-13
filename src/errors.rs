//! 统一错误处理
//!
//! 定义应用程序的错误类型和HTTP响应转换

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// 应用程序错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("认证失败: {0}")]
    Unauthorized(String),

    #[error("资源未找到")]
    NotFound,

    #[error("内部服务器错误: {0}")]
    Internal(String),
    
    #[error("爬虫错误: {0}")]
    Crawler(String),
    
    #[error("Redis错误: {0}")]
    Redis(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "数据库操作失败")
            }
            AppError::Config(ref msg) => {
                tracing::error!("Config error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "配置错误")
            }
            AppError::Unauthorized(ref msg) => {
                (StatusCode::UNAUTHORIZED, msg.as_str())
            }
            AppError::NotFound => {
                (StatusCode::NOT_FOUND, "资源未找到")
            }
            AppError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str())
            }
            AppError::Crawler(ref msg) => {
                tracing::error!("Crawler error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "爬虫服务错误")
            }
            AppError::Redis(ref msg) => {
                tracing::error!("Redis error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "缓存服务错误")
            }
        };

        let body = Json(json!({
            "error": error_message,
            "message": self.to_string(),
        }));

        (status, body).into_response()
    }
}

/// Result类型别名
pub type Result<T> = std::result::Result<T, AppError>;

/// 从 anyhow::Error 转换为 AppError
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Crawler(err.to_string())
    }
}
