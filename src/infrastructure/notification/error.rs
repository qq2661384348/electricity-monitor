//! 通知模块错误类型

use thiserror::Error;

/// 通知错误
#[derive(Debug, Error)]
pub enum NotificationError {
    /// HTTP请求失败
    #[error("HTTP请求失败: {0}")]
    HttpError(#[from] reqwest::Error),
    
    /// QQ机器人API返回错误
    #[error("QQ机器人API返回错误: status={status}, retcode={retcode}, message={message}")]
    ApiError {
        status: String,
        retcode: i32,
        message: String,
    },
    
    /// 消息格式化失败
    #[error("消息格式化失败: {0}")]
    FormatError(String),
    
    /// JSON序列化/反序列化失败
    #[error("JSON处理失败: {0}")]
    JsonError(#[from] sonic_rs::Error),
}

/// 通知结果类型别名
pub type Result<T> = std::result::Result<T, NotificationError>;
