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

    /// 单次同步允许停用的最大活跃房间比例。
    ///
    /// Upay 是房间源真相，但外部接口临时异常时可能返回过小集合。
    /// 这个保护阈值用于阻止一次同步误停用大量生产房间。
    #[serde(default = "default_max_deactivate_ratio")]
    pub max_deactivate_ratio: f64,

    /// 已有房间数据时，允许执行停用逻辑所需的最小源端房间数量。
    #[serde(default = "default_min_source_room_count")]
    pub min_source_room_count: usize,

    /// 爬虫配置
    pub crawler: CrawlerConfig,
}

/// 爬虫配置
#[derive(Debug, Clone, Deserialize)]
pub struct CrawlerConfig {
    /// API Base URL
    #[serde(default = "default_api_url")]
    pub api_url: String,

    /// 学校 / 商户 / 项目的业务标识
    #[serde(default = "default_cid")]
    pub cid: String,

    /// 请求超时时间（秒）
    pub timeout_seconds: u64,

    /// 连接超时时间（秒）
    pub connect_timeout_seconds: u64,

    /// 最大重试次数
    pub max_retries: u32,

    /// 并发数
    pub concurrency: usize,
}

fn default_api_url() -> String {
    "https://upayadmin.gyruibo.cn/UpayManage/Home".to_string()
}

fn default_cid() -> String {
    "689885779152867328".to_string()
}

fn default_max_deactivate_ratio() -> f64 {
    0.5
}

fn default_min_source_room_count() -> usize {
    1000
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        Self {
            api_url: default_api_url(),
            cid: default_cid(),
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
            max_deactivate_ratio: default_max_deactivate_ratio(),
            min_source_room_count: default_min_source_room_count(),
            crawler: CrawlerConfig::default(),
        }
    }
}
