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

    #[error("权限不足")]
    Forbidden,

    #[error("请求参数错误: {0}")]
    BadRequest(String),

    #[error("请求过于频繁: {0}")]
    RateLimited(String),

    #[error("资源未找到")]
    NotFound,

    #[error("用户未添加机器人为好友: {qq_number}")]
    UserNotFriend { qq_number: String },

    #[error("内部服务器错误: {0}")]
    Internal(String),

    #[error("爬虫错误: {0}")]
    Crawler(String),

    #[error("Redis错误: {0}")]
    Redis(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "数据库操作失败")
            }
            AppError::Config(ref msg) => {
                tracing::error!("Config error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "服务配置错误")
            }
            AppError::Unauthorized(ref msg) => (StatusCode::UNAUTHORIZED, msg.as_str()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "权限不足"),
            AppError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::RateLimited(ref msg) => (StatusCode::TOO_MANY_REQUESTS, msg.as_str()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "资源未找到"),
            AppError::UserNotFriend { ref qq_number } => {
                tracing::warn!(qq_number = qq_number, "用户未添加机器人为好友");
                // 返回特殊的 JSON 响应，避免在公开仓库中暴露具体机器人账号
                let body = Json(json!({
                    "error": "USER_NOT_FRIEND",
                    "message": "请先添加当前通知机器人为好友后再发送验证码",
                    "qq_number": qq_number
                }));
                return (StatusCode::BAD_REQUEST, body).into_response();
            }
            AppError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "内部服务器错误")
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
            "message": error_message,
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

/// 从 NotificationError 转换为 AppError
impl From<crate::infrastructure::notification::error::NotificationError> for AppError {
    fn from(err: crate::infrastructure::notification::error::NotificationError) -> Self {
        use crate::infrastructure::notification::error::NotificationError;

        match err {
            NotificationError::UserNotFriend { qq_number } => AppError::UserNotFriend { qq_number },
            NotificationError::HttpError(e) => {
                AppError::Internal(format!("通知服务HTTP请求失败: {}", e))
            }
            NotificationError::ApiError {
                status,
                retcode,
                message,
            } => AppError::Internal(format!(
                "通知服务API错误: status={}, retcode={}, message={}",
                status, retcode, message
            )),
            NotificationError::FormatError(msg) => {
                AppError::Internal(format!("消息格式化失败: {}", msg))
            }
            NotificationError::JsonError(e) => AppError::Internal(format!("JSON处理失败: {}", e)),
        }
    }
}

/// 从 EmailError 转换为 AppError。
impl From<crate::infrastructure::email::EmailError> for AppError {
    fn from(err: crate::infrastructure::email::EmailError) -> Self {
        AppError::Internal(format!("邮件服务错误: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use serde_json::Value;

    async fn response_body(error: AppError) -> (StatusCode, Value) {
        let response = error.into_response();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("错误响应体应可读取");
        let json = serde_json::from_slice(&body).expect("错误响应体应为 JSON");
        (status, json)
    }

    #[tokio::test]
    async fn internal_error_response_does_not_expose_detail() {
        let (status, body) = response_body(AppError::Internal(
            "upstream token abc123 leaked".to_string(),
        ))
        .await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["error"], "内部服务器错误");
        assert_eq!(body["message"], "内部服务器错误");
        assert!(
            !body.to_string().contains("abc123"),
            "公开错误响应不能包含内部细节"
        );
    }

    #[tokio::test]
    async fn database_error_response_uses_stable_public_message() {
        let (status, body) =
            response_body(AppError::Database(diesel::result::Error::NotFound)).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["error"], "数据库操作失败");
        assert_eq!(body["message"], "数据库操作失败");
        assert!(
            !body.to_string().contains("NotFound"),
            "数据库错误细节只能进入日志，不能进入响应体"
        );
    }
}
