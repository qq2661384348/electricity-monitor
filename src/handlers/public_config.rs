//! 前端公开运行时配置处理器

use axum::{response::IntoResponse, Json};
use serde::Serialize;

use crate::config::AppConfig;

#[derive(Debug, Serialize)]
pub struct PublicNotificationConfig {
    pub qq_bot_public_qq_number: String,
    pub admin_qq_number: String,
}

#[derive(Debug, Serialize)]
pub struct PublicCaptchaConfig {
    pub api_url: String,
    pub request_timeout_seconds: u64,
    pub token_expire_seconds: u64,
    pub captcha_type: String,
    pub width: u16,
    pub height: u16,
    pub options: u8,
}

#[derive(Debug, Serialize)]
pub struct PublicVerificationConfig {
    pub code_length: usize,
    pub expire_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct PublicConfigResponse {
    pub notification: PublicNotificationConfig,
    pub captcha: PublicCaptchaConfig,
    pub verification: PublicVerificationConfig,
}

impl From<&AppConfig> for PublicConfigResponse {
    fn from(config: &AppConfig) -> Self {
        Self {
            notification: PublicNotificationConfig {
                qq_bot_public_qq_number: config.qq_bot.public_qq_number.trim().to_string(),
                admin_qq_number: config.admin.default_qq_number.trim().to_string(),
            },
            captcha: PublicCaptchaConfig {
                api_url: config.captcha.api_url.clone(),
                request_timeout_seconds: config.captcha.request_timeout_seconds,
                token_expire_seconds: config.captcha.token_expire_seconds,
                captcha_type: config.captcha.captcha_type.clone(),
                width: config.captcha.width,
                height: config.captcha.height,
                options: config.captcha.options,
            },
            verification: PublicVerificationConfig {
                code_length: config.verification.code_length,
                expire_seconds: config.verification.expire_seconds,
            },
        }
    }
}

/// GET /api/public-config
pub async fn public_config() -> impl IntoResponse {
    Json(PublicConfigResponse::from(AppConfig::global()))
}
