//! NapCat HTTP 机器人服务配置

use serde::Deserialize;

/// NapCat HTTP 机器人服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct QQBotConfig {
    /// NapCat action API 地址
    pub api_url: String,

    /// Bearer Token
    pub bearer_token: String,

    /// Bearer Token secret file 路径
    #[serde(default)]
    pub bearer_token_file: Option<String>,

    /// 请求超时（秒）
    pub timeout_seconds: u64,
}

impl Default for QQBotConfig {
    fn default() -> Self {
        Self {
            api_url: "http://127.0.0.1:3000/send_private_msg".to_string(),
            bearer_token: String::new(),
            bearer_token_file: None,
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
