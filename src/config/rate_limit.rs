//! 后台任务限流配置
//!
//! 这里的可配置项仍只控制电费插入子线程和 send_flag 查询子线程。
//! 登录验证码发送的公开入口使用代码内固定安全阈值，避免误配置为无限制后触达 QQ/SMTP。

use serde::Deserialize;

/// 限流配置
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    /// 每秒最多插入次数（仅限制电费插入子线程）
    pub insert_per_second: u32,

    /// 每秒最多查询次数（仅限制send_flag查询子线程）
    pub query_per_second: u32,
}

impl RateLimitConfig {
    /// 获取插入操作的窗口大小（秒）
    pub fn insert_window_seconds(&self) -> u64 {
        1
    }

    /// 获取查询操作的窗口大小（秒）
    pub fn query_window_seconds(&self) -> u64 {
        1
    }
}
