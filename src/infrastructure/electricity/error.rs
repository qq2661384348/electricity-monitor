//! 电费获取模块错误定义

use thiserror::Error;

/// 电费获取错误类型
#[derive(Error, Debug)]
pub enum ElectricityFetchError {
    /// URL 前缀格式无效
    #[error("无效的 URL 前缀: {0}")]
    InvalidUrlPrefix(String),

    /// 网络请求失败
    #[error("网络请求失败: {0}")]
    NetworkError(#[from] reqwest::Error),

    /// 数据解析失败
    #[error("数据解析失败")]
    ParseError,

    /// 请求超时
    #[error("请求超时")]
    Timeout,

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),

    /// 房间不存在/无效
    #[error("房间不存在")]
    RoomNotFound,
}

/// Result 类型别名
pub type Result<T> = std::result::Result<T, ElectricityFetchError>;
