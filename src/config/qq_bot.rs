//! QQ机器人配置

use serde::Deserialize;

/// QQ机器人配置
#[derive(Debug, Clone, Deserialize)]
pub struct QQBotConfig {
    /// QQ机器人API地址
    pub api_url: String,

    /// Bearer Token
    pub bearer_token: String,

    /// 请求超时（秒）
    pub timeout_seconds: u64,
}

impl Default for QQBotConfig {
    fn default() -> Self {
        Self {
            api_url: "http://47.92.117.121:3000/send_private_msg".to_string(),
            bearer_token: String::new(),
            timeout_seconds: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = QQBotConfig::default();
        assert_eq!(config.timeout_seconds, 10);
        assert!(config.api_url.contains("send_private_msg"));
    }
}
