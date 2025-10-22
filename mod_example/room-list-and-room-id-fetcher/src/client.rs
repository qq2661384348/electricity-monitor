//! HTTP 客户端封装
//!
//! 提供带重试机制的 HTTP 请求客户端

use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

use crate::models::ApiResponse;
use crate::parser;

/// 房间信息 HTTP 客户端
///
/// 封装了与服务器的 HTTP 通信，包括：
/// - 连接池管理
/// - 超时控制
/// - 自动重试（指数退避）
pub struct RoomClient {
    /// reqwest HTTP 客户端（内置连接池）
    client: Client,

    /// API 基础 URL
    base_url: String,
}

impl RoomClient {
    /// 创建新的客户端实例
    ///
    /// # 配置
    /// - 连接超时: 10秒
    /// - 请求超时: 30秒
    /// - 连接池大小: 50（每个主机）
    ///
    /// # 错误
    /// 如果客户端构建失败返回错误
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(50) // 连接池大小
            .build()
            .context("构建 HTTP 客户端失败")?;

        Ok(Self {
            client,
            base_url: "https://zywxhd02.gxust.edu.cn/Home/GetRoomTree".to_string(),
        })
    }

    /// 发送请求获取房间树数据（带重试）
    ///
    /// # 参数
    /// - `params`: URL 编码的请求参数（例如：`yzm=123&Id=000&level=1`）
    ///
    /// # 重试策略
    /// - 最多重试 3 次
    /// - 指数退避: 100ms → 200ms → 400ms
    ///
    /// # 返回
    /// - `Ok(ApiResponse)`: 解析成功的响应
    /// - `Err`: 请求失败或解析失败的错误
    pub async fn fetch_tree(&self, params: &str) -> Result<ApiResponse> {
        const MAX_RETRIES: u32 = 3;
        let mut attempt = 0;

        loop {
            attempt += 1;

            match self.try_fetch(params).await {
                Ok(resp) => {
                    tracing::debug!("请求成功（尝试 {}/{}）", attempt, MAX_RETRIES);
                    return Ok(resp);
                }
                Err(e) if attempt >= MAX_RETRIES => {
                    return Err(e).context(format!("重试 {} 次后仍然失败", MAX_RETRIES));
                }
                Err(e) => {
                    let delay = Duration::from_millis(100 * 2u64.pow(attempt - 1)); // 指数退避
                    tracing::warn!(
                        "请求失败（尝试 {}/{}），{}ms 后重试: {:?}",
                        attempt,
                        MAX_RETRIES,
                        delay.as_millis(),
                        e
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    /// 尝试单次请求（内部方法）
    #[inline]
    async fn try_fetch(&self, params: &str) -> Result<ApiResponse> {
        tracing::debug!("发送 POST 请求: {}", params);

        // 发送 POST 请求
        let resp = self
            .client
            .post(&self.base_url)
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header("Accept", "application/json, text/javascript, */*; q=0.01")
            .body(params.to_string())
            .send()
            .await
            .context("发送 HTTP 请求失败")?;

        // 检查 HTTP 状态码
        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("HTTP 请求失败，状态码: {}", status);
        }

        // 获取响应文本
        let text = resp.text().await.context("读取响应文本失败")?;

        tracing::debug!("收到响应，长度: {} 字节", text.len());

        // 解析 JSON（处理 BOM + 双重编码）
        let value = parser::safe_parse(&text).context("解析 JSON 失败")?;

        // 反序列化为 ApiResponse
        let api_response: ApiResponse =
            serde_json::from_value(value).context("反序列化 ApiResponse 失败")?;

        Ok(api_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = RoomClient::new();
        assert!(client.is_ok());
    }
}
