//! 房间同步服务配置

use serde::Deserialize;

/// 房间同步服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct RoomSyncConfig {
    /// 是否启用同步服务
    pub enabled: bool,

    /// 同步间隔（小时）
    pub interval_hours: u64,

    /// 默认电费阈值
    pub default_threshold: f32,

    /// 爬虫配置
    pub crawler: CrawlerConfig,
}

/// 爬虫配置
#[derive(Debug, Clone, Deserialize)]
pub struct CrawlerConfig {
    /// API URL
    pub api_url: String,

    /// 请求超时时间（秒）
    pub timeout_seconds: u64,

    /// 连接超时时间（秒）
    pub connect_timeout_seconds: u64,

    /// 最大重试次数
    pub max_retries: u32,

    /// 并发数
    pub concurrency: usize,
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        Self {
            api_url: "https://zywxhd02.gxust.edu.cn/Home/GetRoomTree".to_string(),
            timeout_seconds: 30,
            connect_timeout_seconds: 10,
            max_retries: 3,
            concurrency: 50,
        }
    }
}

impl Default for RoomSyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_hours: 24,
            default_threshold: 100.0,
            crawler: CrawlerConfig::default(),
        }
    }
}
