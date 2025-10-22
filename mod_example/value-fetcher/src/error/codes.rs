//! 错误码映射模块
//!
//! 提供统一的错误码定义和查询功能。
//!
//! # 设计特点
//!
//! - 使用 `#[repr(u8)]` 实现类型安全的整数映射（0-255）
//! - 每个错误码对应一个清晰的描述字符串
//! - 支持双向转换：ErrorCode ↔ u8
//! - 内存占用仅 1 字节
//!
//! # 示例
//!
//! ```
//! use electricity_monitor::ErrorCode;
//!
//! // 转换为整数
//! let code = ErrorCode::NetworkError;
//! assert_eq!(code.as_u8(), 2);
//!
//! // 获取描述
//! assert_eq!(code.description(), "网络请求失败");
//!
//! // 从整数反向查询
//! let code = ErrorCode::from_u8(2).unwrap();
//! assert_eq!(code, ErrorCode::NetworkError);
//! ```

use crate::error::FetchError;

/// 错误码枚举（u8 整数映射，0-255）
///
/// 每个错误类型对应一个唯一的整数码，用于在分离结果中标识失败原因。
///
/// # 错误码列表
///
/// | 错误码 | 名称 | 描述 |
/// |--------|------|------|
/// | 1 | InvalidUrlPrefix | 无效的 URL 前缀 |
/// | 2 | NetworkError | 网络请求失败 |
/// | 3 | ParseError | 数据解析失败 |
/// | 4 | Timeout | 请求超时 |
/// | 5 | Internal | 内部错误 |
/// | 6 | RoomNotFound | 房间不存在/无效 |
///
/// # 内存占用
///
/// 使用 `#[repr(u8)]`，每个错误码仅占用 1 字节内存。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// URL 前缀格式无效（错误码 1）
    ///
    /// 当提供的 URL 前缀不符合预期格式时返回。
    /// 预期格式：`https://example.com/api?roomid=`
    InvalidUrlPrefix = 1,

    /// 网络请求失败（错误码 2）
    ///
    /// 包括连接超时、DNS 解析失败、连接被拒绝等网络层错误。
    NetworkError = 2,

    /// 数据解析失败（错误码 3）
    ///
    /// 当服务器返回的数据无法解析为电费数值时返回。
    /// 可能原因：房间不存在、响应格式错误、数据格式变更等。
    ParseError = 3,

    /// 请求超时（错误码 4）
    ///
    /// 当单个请求超过配置的超时时间（默认 8 秒）时返回。
    Timeout = 4,

    /// 内部错误（错误码 5）
    ///
    /// 不应直接暴露给最终用户的内部实现错误。
    /// 通常表示 bug 或不可恢复的状态。
    Internal = 5,

    /// 房间不存在/无效（错误码 6）
    ///
    /// 当 API 返回业务错误状态（如 BS=-1）时返回。
    /// 表示查询的房间 ID 在系统中不存在或无效。
    RoomNotFound = 6,
}

impl ErrorCode {
    /// 获取错误码的描述字符串
    ///
    /// # 返回
    ///
    /// 返回静态字符串切片，不分配堆内存。
    ///
    /// # 示例
    ///
    /// ```
    /// use electricity_monitor::ErrorCode;
    ///
    /// let code = ErrorCode::NetworkError;
    /// assert_eq!(code.description(), "网络请求失败");
    /// ```
    pub fn description(&self) -> &'static str {
        match self {
            Self::InvalidUrlPrefix => "无效的 URL 前缀",
            Self::NetworkError => "网络请求失败",
            Self::ParseError => "数据解析失败",
            Self::Timeout => "请求超时",
            Self::Internal => "内部错误",
            Self::RoomNotFound => "房间不存在",
        }
    }

    /// 转换为 u8 整数
    ///
    /// # 返回
    ///
    /// 返回错误码对应的整数值（1-255）。
    ///
    /// # 示例
    ///
    /// ```
    /// use electricity_monitor::ErrorCode;
    ///
    /// let code = ErrorCode::ParseError;
    /// assert_eq!(code.as_u8(), 3);
    /// ```
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    /// 从 u8 整数转换为 ErrorCode（反向查询）
    ///
    /// # 参数
    ///
    /// * `code` - 错误码整数（1-255）
    ///
    /// # 返回
    ///
    /// - `Some(ErrorCode)` - 有效的错误码
    /// - `None` - 无效的错误码（未定义）
    ///
    /// # 示例
    ///
    /// ```
    /// use electricity_monitor::ErrorCode;
    ///
    /// let code = ErrorCode::from_u8(2);
    /// assert_eq!(code, Some(ErrorCode::NetworkError));
    ///
    /// let invalid = ErrorCode::from_u8(99);
    /// assert_eq!(invalid, None);
    /// ```
    pub fn from_u8(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::InvalidUrlPrefix),
            2 => Some(Self::NetworkError),
            3 => Some(Self::ParseError),
            4 => Some(Self::Timeout),
            5 => Some(Self::Internal),
            6 => Some(Self::RoomNotFound),
            _ => None,
        }
    }
}

/// 从 `FetchError` 引用转换为 `ErrorCode`
///
/// 将高层级的 `FetchError` 映射到简单的整数错误码。
///
/// # 示例
///
/// ```
/// use electricity_monitor::{FetchError, ErrorCode};
///
/// let error = FetchError::ParseError;
/// let code = ErrorCode::from(&error);
/// assert_eq!(code, ErrorCode::ParseError);
/// ```
impl From<&FetchError> for ErrorCode {
    fn from(error: &FetchError) -> Self {
        match error {
            FetchError::InvalidUrlPrefix(_) => Self::InvalidUrlPrefix,
            FetchError::NetworkError(_) => Self::NetworkError,
            FetchError::ParseError => Self::ParseError,
            FetchError::Timeout => Self::Timeout,
            FetchError::Internal(_) => Self::Internal,
            FetchError::RoomNotFound => Self::RoomNotFound,
        }
    }
}

/// 为 `ErrorCode` 实现 `Display` trait
///
/// 允许直接使用 `println!` 或 `format!` 格式化错误码。
///
/// # 示例
///
/// ```
/// use electricity_monitor::ErrorCode;
///
/// let code = ErrorCode::NetworkError;
/// println!("错误: {}", code);  // 输出: 错误: 网络请求失败
/// ```
impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_values() {
        assert_eq!(ErrorCode::InvalidUrlPrefix as u8, 1);
        assert_eq!(ErrorCode::NetworkError as u8, 2);
        assert_eq!(ErrorCode::ParseError as u8, 3);
        assert_eq!(ErrorCode::Timeout as u8, 4);
        assert_eq!(ErrorCode::Internal as u8, 5);
    }

    #[test]
    fn test_error_code_description() {
        assert_eq!(ErrorCode::InvalidUrlPrefix.description(), "无效的 URL 前缀");
        assert_eq!(ErrorCode::NetworkError.description(), "网络请求失败");
        assert_eq!(ErrorCode::ParseError.description(), "数据解析失败");
        assert_eq!(ErrorCode::Timeout.description(), "请求超时");
        assert_eq!(ErrorCode::Internal.description(), "内部错误");
    }

    #[test]
    fn test_error_code_as_u8() {
        assert_eq!(ErrorCode::NetworkError.as_u8(), 2);
        assert_eq!(ErrorCode::ParseError.as_u8(), 3);
    }

    #[test]
    fn test_error_code_from_u8() {
        assert_eq!(ErrorCode::from_u8(1), Some(ErrorCode::InvalidUrlPrefix));
        assert_eq!(ErrorCode::from_u8(2), Some(ErrorCode::NetworkError));
        assert_eq!(ErrorCode::from_u8(3), Some(ErrorCode::ParseError));
        assert_eq!(ErrorCode::from_u8(4), Some(ErrorCode::Timeout));
        assert_eq!(ErrorCode::from_u8(5), Some(ErrorCode::Internal));
        assert_eq!(ErrorCode::from_u8(99), None);
        assert_eq!(ErrorCode::from_u8(0), None);
    }

    #[test]
    fn test_error_code_from_fetch_error() {
        let error = FetchError::ParseError;
        let code = ErrorCode::from(&error);
        assert_eq!(code, ErrorCode::ParseError);
        assert_eq!(code.as_u8(), 3);
    }

    #[test]
    fn test_error_code_roundtrip() {
        let original = ErrorCode::NetworkError;
        let code_u8 = original.as_u8();
        let restored = ErrorCode::from_u8(code_u8).unwrap();
        assert_eq!(original, restored);
    }
}
