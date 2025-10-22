//! 统一错误处理模块
//!
//! 定义了应用程序中所有可能的错误类型，使用 `thiserror` 简化错误定义

// 错误码映射模块
pub mod codes;

// 导出公开类型
pub use codes::ErrorCode;

use thiserror::Error;

/// 电费监控应用的统一错误类型
///
/// # 示例
///
/// ```no_run
/// use electricity_monitor::error::ElectricityError;
///
/// fn may_fail() -> Result<(), ElectricityError> {
///     Err(ElectricityError::ConfigError("配置文件不存在".to_string()))
/// }
/// ```
#[derive(Error, Debug)]
pub enum ElectricityError {
    /// 配置加载或解析错误
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// HTTP 请求相关错误
    #[error("HTTP 请求失败: {0}")]
    HttpError(#[from] reqwest::Error),

    /// 数据解析错误（正则匹配失败等）
    #[error("数据解析失败: {0}")]
    ParseError(String),

    /// IO 操作错误
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    /// 其他未分类错误
    #[error("未知错误: {0}")]
    Other(String),
}

/// 应用程序通用的 Result 类型别名
///
/// 将 `std::result::Result` 的错误类型固定为 `ElectricityError`，
/// 简化函数签名
pub type Result<T> = std::result::Result<T, ElectricityError>;

/// 电费查询模块的公开错误类型
///
/// 用于库的公开 API，提供清晰的错误分类和详细的错误信息。
///
/// # 示例
///
/// ```no_run
/// use electricity_monitor::FetchError;
/// use std::collections::HashMap;
///
/// async fn query_rooms(room_ids: Vec<u32>) -> Result<HashMap<u32, f64>, FetchError> {
///     // ... 查询逻辑
///     # Ok(HashMap::new())
/// }
/// ```
#[derive(Error, Debug)]
pub enum FetchError {
    /// URL 前缀格式无效
    ///
    /// 当提供的 URL 前缀不符合预期格式时返回此错误。
    /// 预期格式：`https://example.com/api?roomid=`（必须以 `?roomid=` 结尾）
    #[error("无效的 URL 前缀: {0}")]
    InvalidUrlPrefix(String),

    /// 网络请求失败
    ///
    /// 包括连接超时、DNS 解析失败、连接被拒绝等网络层错误。
    #[error("网络请求失败: {0}")]
    NetworkError(#[from] reqwest::Error),

    /// 数据解析失败
    ///
    /// 当服务器返回的数据无法解析为电费数值时返回此错误。
    /// 可能原因：房间不存在、响应格式错误、数据格式变更等。
    #[error("数据解析失败")]
    ParseError,

    /// 请求超时
    ///
    /// 当单个请求超过配置的超时时间（默认 8 秒）时返回此错误。
    #[error("请求超时（8秒）")]
    Timeout,

    /// 内部错误
    ///
    /// 不应直接暴露给最终用户的内部实现错误。
    /// 通常表示 bug 或不可恢复的状态。
    #[error("内部错误: {0}")]
    Internal(String),

    /// 房间不存在/无效
    ///
    /// 当 API 返回业务错误状态（如 BS=-1）时返回此错误。
    /// 表示查询的房间 ID 在系统中不存在或无效。
    #[error("房间不存在")]
    RoomNotFound,
}

impl From<ElectricityError> for FetchError {
    fn from(err: ElectricityError) -> Self {
        match err {
            ElectricityError::HttpError(e) => FetchError::NetworkError(e),
            ElectricityError::ParseError(_) => FetchError::ParseError,
            ElectricityError::ConfigError(msg) => FetchError::InvalidUrlPrefix(msg),
            _ => FetchError::Internal(err.to_string()),
        }
    }
}
