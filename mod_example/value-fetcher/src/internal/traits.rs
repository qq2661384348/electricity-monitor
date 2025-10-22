//! 内部 trait 定义（仅 crate 内可见）
//!
//! 定义核心抽象接口以提升可测试性和解耦度。
//! 所有 trait 仅在 crate 内部可见，不影响公共 API。

use crate::error::Result;
use async_trait::async_trait;

/// HTTP 客户端 trait（内部抽象）
///
/// 提供异步 HTTP GET 请求的抽象接口，支持依赖注入和 Mock 测试。
///
/// # 实现
///
/// - `ReqwestAsyncClient`: 生产环境实现（基于 reqwest）
/// - `MockHttpClient`: 测试环境 Mock 实现（使用 mockall）
///
/// # 示例
///
/// ```no_run
/// use async_trait::async_trait;
/// use electricity_monitor::error::Result;
///
/// struct MyClient;
///
/// #[async_trait]
/// impl HttpClient for MyClient {
///     async fn get(&self, url: &str) -> Result<String> {
///         Ok("mock response".to_string())
///     }
/// }
/// ```
#[async_trait]
pub(crate) trait HttpClient: Send + Sync {
    /// 执行异步 HTTP GET 请求
    ///
    /// # 参数
    ///
    /// * `url` - 请求的完整 URL
    ///
    /// # 返回
    ///
    /// - `Ok(String)` - HTTP 响应体文本
    /// - `Err(ElectricityError)` - 网络错误或 HTTP 错误
    ///
    /// # 错误
    ///
    /// - `ElectricityError::HttpError` - HTTP 请求失败
    /// - `ElectricityError::NetworkError` - 网络连接失败
    async fn get(&self, url: &str) -> Result<String>;
}

/// 数据解析器 trait（内部抽象）
///
/// 提供从 HTTP 响应中提取电费数据的抽象接口。
///
/// # 实现
///
/// - `ElectricityParser`: 生产环境实现（基于字符串搜索）
/// - `MockParser`: 测试环境 Mock 实现
///
/// # 示例
///
/// ```no_run
/// struct MyParser;
///
/// impl DataParser for MyParser {
///     fn parse(&self, raw_data: &str) -> Option<String> {
///         Some("45.67".to_string())
///     }
/// }
/// ```
pub(crate) trait DataParser: Send + Sync {
    /// 解析 HTTP 响应，提取电费值
    ///
    /// # 参数
    ///
    /// * `raw_data` - HTTP 响应体（JSON 字符串）
    ///
    /// # 返回
    ///
    /// - `Some(String)` - 成功提取的电费值（或 "ROOM_NOT_FOUND"）
    /// - `None` - 解析失败
    ///
    /// # 特殊值
    ///
    /// - `"ROOM_NOT_FOUND"` - 房间不存在（BS=-1）
    fn parse(&self, raw_data: &str) -> Option<String>;
}

/// URL 构建器 trait（内部抽象）
///
/// 提供 URL 参数替换的抽象接口，支持高性能 roomid 替换。
///
/// # 实现
///
/// - `UrlBuilder`: 生产环境实现（支持 FastPrefix 优化）
/// - `MockBuilder`: 测试环境 Mock 实现
///
/// # 示例
///
/// ```no_run
/// struct MyBuilder;
///
/// impl UrlBuilder for MyBuilder {
///     fn with_roomid(&self, roomid: &str) -> String {
///         format!("https://example.com?roomid={}", roomid)
///     }
///     
///     fn with_roomid_u32(&self, roomid: u32) -> String {
///         format!("https://example.com?roomid={}", roomid)
///     }
/// }
/// ```
pub(crate) trait UrlBuilder: Send + Sync {
    /// 使用字符串 roomid 构建 URL
    ///
    /// # 参数
    ///
    /// * `roomid` - 房间 ID（字符串形式）
    ///
    /// # 返回
    ///
    /// 完整的请求 URL
    fn with_roomid(&self, roomid: &str) -> String;

    /// 使用 u32 roomid 构建 URL（优化版本）
    ///
    /// # 参数
    ///
    /// * `roomid` - 房间 ID（u32 类型）
    ///
    /// # 返回
    ///
    /// 完整的请求 URL
    ///
    /// # 性能
    ///
    /// FastPrefix 模式下约 37ns，Generic 模式约 549ns
    fn with_roomid_u32(&self, roomid: u32) -> String;
}
