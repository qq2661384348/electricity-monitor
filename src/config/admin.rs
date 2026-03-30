//! 管理员配置

use serde::Deserialize;

/// 管理员配置
#[derive(Debug, Clone, Deserialize)]
pub struct AdminConfig {
    /// 默认管理员 QQ 号占位值
    pub default_qq_number: String,
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self {
            default_qq_number: "100000001".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AdminConfig::default();
        assert_eq!(config.default_qq_number, "100000001");
    }
}
