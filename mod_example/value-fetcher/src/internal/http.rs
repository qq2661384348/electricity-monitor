//! HTTP 客户端实现（内部模块）
//!
//! 基于 reqwest async 封装，提供高性能异步 HTTP 请求。
//! 连接池优化：100 连接/host，90秒超时，TCP keepalive 60秒。

use crate::error::{ElectricityError, Result};
use crate::internal::traits::HttpClient;
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

/// 异步 HTTP 客户端（内部使用）
///
/// # 特性
/// - 禁用 SSL 验证（测试环境）
/// - 超时 30 秒
/// - 连接池优化（100 连接/host）
pub(crate) struct ReqwestAsyncClient {
    client: Client,
}

impl ReqwestAsyncClient {
    /// 创建 HTTP 客户端
    ///
    /// # 参数
    /// * `disable_ssl_verify` - 是否禁用 SSL 证书验证
    ///
    /// # 连接池配置
    /// - 100 空闲连接/host
    /// - 超时 90 秒
    /// - TCP keepalive 60 秒
    pub(crate) fn new(disable_ssl_verify: bool) -> Result<Self> {
        let client = Client::builder()
            .danger_accept_invalid_certs(disable_ssl_verify)
            .timeout(Duration::from_secs(30))
            // 连接池优化（关键！）
            .pool_max_idle_per_host(100)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .build()?;

        Ok(Self { client })
    }

    /// 执行 GET 请求
    pub(crate) async fn get(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await?;

        // 检查响应状态
        if !response.status().is_success() {
            return Err(ElectricityError::HttpError(
                response.error_for_status().unwrap_err(),
            ));
        }

        let text = response.text().await?;
        Ok(text)
    }
}

/// 为 ReqwestAsyncClient 实现 HttpClient trait
#[async_trait]
impl HttpClient for ReqwestAsyncClient {
    async fn get(&self, url: &str) -> Result<String> {
        self.get(url).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ReqwestAsyncClient::new(true);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_async_get() {
        let client = ReqwestAsyncClient::new(true).unwrap();
        // 这里不实际调用，只验证编译
        let _ = client;
    }
}
