//! 第三方验证码校验服务
//!
//! 采用"客户端直连获取、网关代理校验"架构
//! 第三方API: https://v2.xxapi.cn/api/captcha

use crate::errors::{AppError, Result};
use crate::infrastructure::RedisPool;
use redis::AsyncCommands;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// 验证码类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptchaType {
    String, // 字符验证码
    Math,   // 算数验证码
    Digit,  // 数字验证码
}

/// 第三方验证码校验请求
#[derive(Debug, Serialize)]
pub struct ThirdPartyVerifyRequest {
    pub id: String,
    pub key: String,
    #[serde(rename = "type")]
    pub captcha_type: String,
}

/// 第三方验证码响应
#[derive(Debug, Deserialize)]
pub struct ThirdPartyResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<String>,
}

/// 验证码校验服务
pub struct CaptchaVerificationService {
    /// HTTP客户端
    http_client: Client,

    /// Redis连接池（用于存储一次性token）
    redis_pool: RedisPool,

    /// 第三方API地址
    api_url: String,
}

impl CaptchaVerificationService {
    /// 创建验证码校验服务
    pub fn new(redis_pool: RedisPool) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            redis_pool,
            api_url: "https://v2.xxapi.cn/api/captcha".to_string(),
        }
    }

    /// 校验验证码（网关代理）
    ///
    /// # 参数
    /// * `id` - 验证码ID
    /// * `key` - 用户输入的答案
    /// * `captcha_type` - 验证码类型
    ///
    /// # 返回
    /// 成功返回一次性token，失败返回错误
    pub async fn verify_captcha(
        &self,
        id: String,
        key: String,
        captcha_type: CaptchaType,
    ) -> Result<String> {
        tracing::debug!(
            captcha_id = %id,
            captcha_type = ?captcha_type,
            "开始校验验证码"
        );

        // 构建请求URL
        let url = format!(
            "{}?id={}&key={}&type={}",
            self.api_url,
            urlencoding::encode(&id),
            urlencoding::encode(&key),
            match captcha_type {
                CaptchaType::String => "string",
                CaptchaType::Math => "math",
                CaptchaType::Digit => "digit",
            }
        );

        // 发送GET请求到第三方API
        let response = self.http_client.get(&url).send().await.map_err(|e| {
            tracing::error!(error = %e, "验证码校验请求失败");
            AppError::Internal(format!("验证码服务请求失败: {}", e))
        })?;

        // 检查HTTP状态码
        if response.status() != StatusCode::OK {
            tracing::warn!(
                status = %response.status(),
                "验证码校验HTTP状态异常"
            );
            return Err(AppError::Internal("验证码服务暂时不可用".to_string()));
        }

        // 解析响应
        let body = response.text().await.map_err(|e| {
            tracing::error!(error = %e, "读取验证码响应失败");
            AppError::Internal("验证码服务响应异常".to_string())
        })?;

        let api_response: ThirdPartyResponse = sonic_rs::from_str(&body).map_err(|e| {
            tracing::error!(error = %e, body = %body, "解析验证码响应失败");
            AppError::Internal("验证码服务响应格式错误".to_string())
        })?;

        // 检查验证结果
        if api_response.code != 200 {
            tracing::info!(
                captcha_id = %id,
                code = api_response.code,
                msg = %api_response.msg,
                "验证码验证失败"
            );
            return Err(AppError::Unauthorized("验证码错误或已过期".to_string()));
        }

        // 验证成功，生成一次性token
        let token = Uuid::new_v4().to_string();

        // 存储token到Redis，有效期60秒
        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| AppError::Redis(format!("获取Redis连接失败: {}", e)))?;

        let redis_key = format!("captcha:token:{}", token);
        conn.set_ex::<_, _, ()>(&redis_key, "valid", 60)
            .await
            .map_err(|e| AppError::Redis(format!("存储验证码token失败: {}", e)))?;

        tracing::info!(
            captcha_id = %id,
            token = %token,
            "验证码验证成功，生成一次性token"
        );

        Ok(token)
    }

    /// 验证并消费token
    ///
    /// # 参数
    /// * `token` - 一次性token
    ///
    /// # 返回
    /// token有效返回true，无效或已使用返回false
    pub async fn verify_and_consume_token(&self, token: &str) -> Result<bool> {
        let redis_key = format!("captcha:token:{}", token);

        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| AppError::Redis(format!("获取Redis连接失败: {}", e)))?;

        // 使用GETDEL原子操作，获取并删除token（确保只能使用一次）
        let exists: Option<String> = conn
            .get_del(&redis_key)
            .await
            .map_err(|e| AppError::Redis(format!("验证token失败: {}", e)))?;

        match exists {
            Some(_) => {
                tracing::info!(token = %token, "Token验证成功并已消费");
                Ok(true)
            }
            None => {
                tracing::warn!(token = %token, "Token不存在或已过期");
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_captcha_type_serialization() {
        let math_type = CaptchaType::Math;
        let json = sonic_rs::to_string(&math_type).unwrap();
        assert_eq!(json, r#""math""#);

        let string_type = CaptchaType::String;
        let json = sonic_rs::to_string(&string_type).unwrap();
        assert_eq!(json, r#""string""#);
    }

    #[test]
    fn test_third_party_response_deserialization() {
        let json = r#"{"code":200,"msg":"数据请求成功","data":"验证成功"}"#;
        let response: ThirdPartyResponse = sonic_rs::from_str(json).unwrap();
        assert_eq!(response.code, 200);
        assert_eq!(response.msg, "数据请求成功");
        assert_eq!(response.data.unwrap(), "验证成功");
    }
}
