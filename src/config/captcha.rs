//! 第三方图形验证码配置

use serde::{Deserialize, Serialize};

/// 第三方图形验证码配置。
///
/// 客户端负责直连生成验证码图片，后端负责代理校验与签发一次性 token。
/// 两端必须共享同一组非敏感参数，避免前端硬编码后和后端校验网关漂移。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CaptchaConfig {
    /// 第三方验证码 API 地址
    pub api_url: String,

    /// 第三方验证码请求超时（秒）
    pub request_timeout_seconds: u64,

    /// 后端签发的一次性 captcha token 过期时间（秒）
    pub token_expire_seconds: u64,

    /// 前端生成验证码时使用的类型
    pub captcha_type: String,

    /// 前端生成验证码时使用的宽度
    pub width: u16,

    /// 前端生成验证码时使用的高度
    pub height: u16,

    /// 前端生成验证码时使用的难度等级
    pub options: u8,
}

impl Default for CaptchaConfig {
    fn default() -> Self {
        Self {
            api_url: "https://v2.xxapi.cn/api/captcha".to_string(),
            request_timeout_seconds: 5,
            token_expire_seconds: 60,
            captcha_type: "math".to_string(),
            width: 300,
            height: 100,
            options: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CaptchaConfig::default();
        assert_eq!(config.api_url, "https://v2.xxapi.cn/api/captcha");
        assert_eq!(config.request_timeout_seconds, 5);
        assert_eq!(config.token_expire_seconds, 60);
        assert_eq!(config.captcha_type, "math");
        assert_eq!(config.width, 300);
        assert_eq!(config.height, 100);
        assert_eq!(config.options, 2);
    }
}
