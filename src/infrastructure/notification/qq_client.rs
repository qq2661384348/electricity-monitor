//! QQ机器人客户端

use super::error::{NotificationError, Result};
use super::message_builder::MessageBuilder;
use crate::config::QQBotConfig;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// QQ API响应
#[derive(Debug, Deserialize, Serialize)]
pub struct QQMessageResponse {
    /// 响应状态
    pub status: String,
    
    /// 返回码
    pub retcode: i32,
    
    /// 响应数据
    #[serde(default)]
    pub data: Option<QQMessageData>,
    
    /// 错误消息
    #[serde(default)]
    pub message: Option<String>,
}

/// QQ消息数据
#[derive(Debug, Deserialize, Serialize)]
pub struct QQMessageData {
    /// 消息ID
    pub message_id: i32,
}

/// QQ机器人客户端
#[derive(Clone)]
pub struct QQClient {
    /// HTTP客户端
    client: Client,
    
    /// 配置
    config: QQBotConfig,
}

impl QQClient {
    /// 创建QQ客户端
    /// 
    /// # 参数
    /// * `config` - QQ机器人配置
    /// 
    /// # 返回
    /// QQ客户端实例
    pub fn new(config: QQBotConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(NotificationError::HttpError)?;
        
        Ok(Self { client, config })
    }
    
    /// 发送私聊消息
    /// 
    /// # 参数
    /// * `user_id` - QQ号
    /// * `message` - 消息文本
    /// 
    /// # 返回
    /// API响应
    /// 
    /// # 错误
    /// - HTTP请求失败
    /// - API返回错误
    pub async fn send_private_message(&self, user_id: &str, message: &str) -> Result<QQMessageResponse> {
        tracing::debug!(
            user_id = user_id,
            message_preview = &message[..message.len().min(50)],
            "发送QQ私聊消息"
        );
        
        // 构建请求体
        let body = MessageBuilder::build_api_request_body(user_id, message);
        
        // 发送请求
        let response = self
            .client
            .post(&self.config.api_url)
            .header("Authorization", format!("Bearer {}", self.config.bearer_token))
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await?;
        
        // 检查HTTP状态码
        let status_code = response.status();
        if !status_code.is_success() {
            tracing::error!(
                status_code = status_code.as_u16(),
                "QQ API HTTP请求失败"
            );
            return Err(NotificationError::ApiError {
                status: "http_error".to_string(),
                retcode: status_code.as_u16() as i32,
                message: format!("HTTP状态码: {}", status_code),
            });
        }
        
        // 解析响应
        let response_text = response.text().await?;
        let api_response: QQMessageResponse = sonic_rs::from_str(&response_text)?;
        
        // 检查API返回码
        if api_response.retcode != 0 {
            tracing::error!(
                retcode = api_response.retcode,
                message = ?api_response.message,
                "QQ API返回错误"
            );
            
            // 特殊处理: retcode=200 且消息包含"无法获取用户信息" -> 用户未添加好友
            if api_response.retcode == 200 {
                if let Some(ref msg) = api_response.message {
                    if msg.contains("无法获取用户信息") {
                        tracing::warn!(
                            user_id = user_id,
                            "用户未添加机器人为好友"
                        );
                        return Err(NotificationError::UserNotFriend {
                            qq_number: user_id.to_string(),
                        });
                    }
                }
            }
            
            return Err(NotificationError::ApiError {
                status: api_response.status.clone(),
                retcode: api_response.retcode,
                message: api_response.message.clone().unwrap_or_default(),
            });
        }
        
        tracing::info!(
            user_id = user_id,
            message_id = ?api_response.data.as_ref().map(|d| d.message_id),
            "QQ消息发送成功"
        );
        
        Ok(api_response)
    }
    
    /// 发送验证码
    /// 
    /// # 参数
    /// * `qq_number` - QQ号
    /// * `code` - 验证码
    /// 
    /// # 返回
    /// API响应
    pub async fn send_verification_code(&self, qq_number: &str, code: &str) -> Result<QQMessageResponse> {
        let message = MessageBuilder::build_verification_code_message(code);
        self.send_private_message(qq_number, &message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qq_client_creation() {
        let config = QQBotConfig {
            api_url: "http://test.com/api".to_string(),
            bearer_token: "test_token".to_string(),
            timeout_seconds: 10,
        };
        
        let client = QQClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_qq_message_response_deserialization() {
        let json = r#"{"status":"ok","retcode":0,"data":{"message_id":123}}"#;
        let response: QQMessageResponse = sonic_rs::from_str(json).unwrap();
        assert_eq!(response.status, "ok");
        assert_eq!(response.retcode, 0);
        assert_eq!(response.data.unwrap().message_id, 123);
    }

    #[test]
    fn test_qq_error_response_deserialization() {
        let json = r#"{"status":"failed","retcode":100,"message":"发送失败"}"#;
        let response: QQMessageResponse = sonic_rs::from_str(json).unwrap();
        assert_eq!(response.status, "failed");
        assert_eq!(response.retcode, 100);
        assert_eq!(response.message.unwrap(), "发送失败");
    }
}
