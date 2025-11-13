//! 验证码配置

use serde::Deserialize;

/// 验证码配置
#[derive(Debug, Clone, Deserialize)]
pub struct VerificationConfig {
    /// 验证码长度
    pub code_length: usize,
    
    /// 验证码过期时间（秒）
    pub expire_seconds: u64,
    
    /// Redis键前缀
    pub redis_key_prefix: String,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            code_length: 6,
            expire_seconds: 300,  // 5分钟
            redis_key_prefix: "verify".to_string(),
        }
    }
}

impl VerificationConfig {
    /// 生成Redis键
    pub fn redis_key(&self, qq_number: &str) -> String {
        format!("{}:{}", self.redis_key_prefix, qq_number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = VerificationConfig::default();
        assert_eq!(config.code_length, 6);
        assert_eq!(config.expire_seconds, 300);
    }

    #[test]
    fn test_redis_key() {
        let config = VerificationConfig::default();
        let key = config.redis_key("123456");
        assert_eq!(key, "verify:123456");
    }
}
