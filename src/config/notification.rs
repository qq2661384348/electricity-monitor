//! 通知配置

use serde::Deserialize;

/// 通知配置
#[derive(Debug, Clone, Deserialize)]
pub struct NotificationConfig {
    /// 并发发送通知数量限制
    pub concurrent_send_limit: usize,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            concurrent_send_limit: 10,
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
    }
}
