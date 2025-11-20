//! 通知配置

use serde::Deserialize;

/// 通知配置
#[derive(Debug, Clone, Deserialize)]
pub struct NotificationConfig {
    /// 并发发送通知数量限制
    pub concurrent_send_limit: usize,
    
    /// 查询间隔（秒）
    /// 
    /// 通知服务查询需要发送通知的房间的间隔时间
    /// 默认: 60秒
    #[serde(default = "default_query_interval_secs")]
    pub query_interval_secs: u64,
    
    /// 防抖观察期（秒）
    /// 
    /// 当房间电费恢复到阈值以上后，需要等待此时长才能重置通知状态
    /// 用于防止电费在阈值附近抖动导致的重复通知
    /// 默认: 3600秒（1小时）
    #[serde(default = "default_debounce_period_secs")]
    pub debounce_period_secs: u64,
    
    /// 恢复监控间隔（秒）
    /// 
    /// 监控任务查询恢复中房间的间隔时间
    /// 默认: 300秒（5分钟）
    #[serde(default = "default_recovery_monitor_interval_secs")]
    pub recovery_monitor_interval_secs: u64,
}

/// 默认查询间隔：60秒
fn default_query_interval_secs() -> u64 {
    60
}

/// 默认防抖观察期：1小时
fn default_debounce_period_secs() -> u64 {
    3600
}

/// 默认恢复监控间隔：5分钟
fn default_recovery_monitor_interval_secs() -> u64 {
    300
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            concurrent_send_limit: 10,
            query_interval_secs: default_query_interval_secs(),
            debounce_period_secs: default_debounce_period_secs(),
            recovery_monitor_interval_secs: default_recovery_monitor_interval_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NotificationConfig::default();
        assert_eq!(config.concurrent_send_limit, 10);
        assert_eq!(config.query_interval_secs, 60);
        assert_eq!(config.debounce_period_secs, 3600);
        assert_eq!(config.recovery_monitor_interval_secs, 300);
    }
}
