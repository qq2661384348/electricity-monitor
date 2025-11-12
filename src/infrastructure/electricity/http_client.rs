//! HTTP 客户端实现
//!
//! 基于 reqwest async 封装，提供高性能异步 HTTP 请求

use super::error::Result;
use reqwest::Client;
use std::time::Duration;

/// 异步 HTTP 客户端
///
/// # 特性
/// - 禁用 SSL 验证（测试环境）
/// - 超时 30 秒
/// - 连接池优化（100 连接/host）
pub struct ReqwestAsyncClient {
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
    pub fn new(disable_ssl_verify: bool) -> Result<Self> {
        let client = Client::builder()
            .danger_accept_invalid_certs(disable_ssl_verify)
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(100)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .build()?;

        Ok(Self { client })
    }

    /// 执行 GET 请求
    pub async fn get(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await?;

        // 检查响应状态
        if !response.status().is_success() {
            return Err(super::error::ElectricityFetchError::NetworkError(
                response.error_for_status().unwrap_err(),
            ));
        }

        let text = response.text().await?;
        Ok(text)
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
}
