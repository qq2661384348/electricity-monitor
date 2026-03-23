//! 限流配置
//!
//! 限流仅针对电费插入子线程和send_flag查询子线程
//! 目的是防止这两个后台任务过多并发导致主业务卡顿

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
