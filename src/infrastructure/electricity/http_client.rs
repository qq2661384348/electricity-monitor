//! HTTP 客户端实现
//!
//! 基于 reqwest async 封装，提供高性能异步 HTTP 请求

use super::error::Result;
use crate::infrastructure::external::{build_reqwest_client, HttpClientConfig};
use reqwest::Client;
use std::time::Duration;

/// 异步 HTTP 客户端
///
/// # 特性
/// - 使用系统根证书校验 HTTPS，避免生产电费数据被中间人篡改
/// - 超时 30 秒
/// - 连接池优化（100 连接/host）
pub struct ReqwestAsyncClient {
    client: Client,
}

impl ReqwestAsyncClient {
    /// 创建 HTTP 客户端
    ///
    /// # 连接池配置
    /// - 100 空闲连接/host
    /// - 超时 90 秒
    /// - TCP keepalive 60 秒
    pub fn new() -> Result<Self> {
        let client = build_reqwest_client(&HttpClientConfig {
            timeout: Some(Duration::from_secs(30)),
            danger_accept_invalid_certs: false,
            pool_max_idle_per_host: Some(100),
            pool_idle_timeout: Some(Duration::from_secs(90)),
            tcp_keepalive: Some(Duration::from_secs(60)),
            ..Default::default()
        })?;

        Ok(Self { client })
    }

    /// 执行 GET 请求
    pub async fn get(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await?;

        // 检查响应状态
        if !response.status().is_success() {
            tracing::error!(
                external_dependency = "electricity_fetcher",
                status = response.status().as_u16(),
                "外部 HTTP 请求失败"
            );
            return Err(super::error::ElectricityFetchError::NetworkError(
                response.error_for_status().unwrap_err(),
            ));
        }

        let text = response.text().await?;
        tracing::debug!(
            external_dependency = "electricity_fetcher",
            body_length = text.len(),
            "外部 HTTP 响应成功"
        );
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ReqwestAsyncClient::new();
        assert!(client.is_ok());
    }
}
