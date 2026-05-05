//! 验证码处理器
//!
//! 处理第三方验证码校验相关的HTTP请求

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::domain::services::captcha_verification::{CaptchaType, CaptchaVerificationService};
use crate::errors::{AppError, Result};
use crate::state::AppState;

/// 验证码校验请求
#[derive(Debug, Deserialize, Validate)]
pub struct VerifyCaptchaRequest {
    /// 验证码ID
    #[validate(length(min = 1, max = 100, message = "验证码ID无效"))]
    pub id: String,

    /// 用户输入的答案
    #[validate(length(min = 1, max = 50, message = "答案长度无效"))]
    pub key: String,

    /// 验证码类型
    #[serde(rename = "type")]
    pub captcha_type: String,
}

/// 验证码校验响应（标准化响应）
#[derive(Debug, Serialize)]
pub struct VerifyCaptchaResponse {
    /// 是否成功
    pub success: bool,

    /// 消息
    pub message: String,

    /// 错误码
    pub code: String,

    /// 一次性token（成功时返回）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// 校验验证码
///
/// POST /api/captcha/verify
///
/// 网关代理校验第三方验证码
pub async fn verify_captcha(
    State(state): State<AppState>,
    Json(req): Json<VerifyCaptchaRequest>,
) -> Result<(StatusCode, Json<VerifyCaptchaResponse>)> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::Internal(format!("请求验证失败: {}", e)))?;

    tracing::debug!(
        captcha_id = %req.id,
        captcha_type = %req.captcha_type,
        "收到验证码校验请求"
    );

    // 解析验证码类型
    let captcha_type = match req.captcha_type.to_lowercase().as_str() {
        "math" => CaptchaType::Math,
        "string" => CaptchaType::String,
        "digit" => CaptchaType::Digit,
        _ => {
            tracing::warn!(
                captcha_type = %req.captcha_type,
                "不支持的验证码类型"
            );
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(VerifyCaptchaResponse {
                    success: false,
                    message: "不支持的验证码类型".to_string(),
                    code: "INVALID_TYPE".to_string(),
                    token: None,
                }),
            ));
        }
    };

    // 创建验证码服务
    let captcha_service = CaptchaVerificationService::new(state.redis_pool.clone());

    // 校验验证码
    match captcha_service
        .verify_captcha(req.id.clone(), req.key.clone(), captcha_type)
        .await
    {
        Ok(token) => {
            tracing::info!(
                captcha_id = %req.id,
                "验证码校验成功"
            );

            Ok((
                StatusCode::OK,
                Json(VerifyCaptchaResponse {
                    success: true,
                    message: "验证通过".to_string(),
                    code: "VERIFY_SUCCESS".to_string(),
                    token: Some(token),
                }),
            ))
        }
        Err(AppError::Unauthorized(_)) => {
            tracing::info!(
                captcha_id = %req.id,
                "验证码校验失败"
            );

            Ok((
                StatusCode::BAD_REQUEST,
                Json(VerifyCaptchaResponse {
                    success: false,
                    message: "验证码错误或已过期".to_string(),
                    code: "VERIFY_FAILED".to_string(),
                    token: None,
                }),
            ))
        }
        Err(_) => {
            tracing::error!(
                captcha_id = %req.id,
                "验证码服务异常"
            );

            Ok((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(VerifyCaptchaResponse {
                    success: false,
                    message: "验证服务暂时不可用".to_string(),
                    code: "SERVICE_UNAVAILABLE".to_string(),
                    token: None,
                }),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_request_validation() {
        let valid_request = VerifyCaptchaRequest {
            id: "test_id".to_string(),
            key: "123".to_string(),
            captcha_type: "math".to_string(),
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = VerifyCaptchaRequest {
            id: "".to_string(), // Empty ID
            key: "123".to_string(),
            captcha_type: "math".to_string(),
        };

        assert!(invalid_request.validate().is_err());
    }
}
