//! NapCat HTTP 机器人服务配置

use serde::Deserialize;

/// NapCat HTTP 机器人服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct QQBotConfig {
    /// NapCat action API 地址
    pub api_url: String,

    /// 对用户公开展示的机器人 QQ 号
    #[serde(default)]
    pub public_qq_number: String,

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
            api_url: String::new(),
            public_qq_number: String::new(),
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
        assert!(config.api_url.is_empty());
        assert!(config.public_qq_number.is_empty());
        assert!(config.bearer_token.is_empty());
    }
}
