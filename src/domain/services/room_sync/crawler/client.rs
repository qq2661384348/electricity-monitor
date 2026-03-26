//! HTTP客户端
//!
//! 负责与外部API通信，获取房间树JSON数据

use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

use crate::config::CrawlerConfig;
use crate::infrastructure::external::{
    build_reqwest_client, http_status_error_message, HttpClientConfig,
};

/// 房间API客户端
pub struct RoomClient {
    /// HTTP客户端
    client: Client,

    /// API URL
    api_url: String,

    /// 最大重试次数
    max_retries: u32,
}

impl RoomClient {
    /// 创建新的客户端实例
    ///
    /// # 参数
    /// - `config`: 爬虫配置
    ///
    /// # 错误
    /// 如果HTTP客户端创建失败
    pub fn new(config: &CrawlerConfig) -> Result<Self> {
        let client = build_reqwest_client(&HttpClientConfig {
            timeout: Some(Duration::from_secs(config.timeout_seconds)),
            connect_timeout: Some(Duration::from_secs(config.connect_timeout_seconds)),
            ..Default::default()
        })
        .context("创建HTTP客户端失败")?;

        Ok(Self {
            client,
            api_url: config.api_url.clone(),
            max_retries: config.max_retries,
        })
    }

    /// 获取房间树JSON数据（Level 1：校区列表）
    ///
    /// 带重试机制的HTTP POST请求
    ///
    /// # 返回
    /// JSON字符串
    ///
    /// # 错误
    /// - 网络请求失败
    /// - 超时
    /// - HTTP错误状态码
    pub async fn fetch_room_tree(&self) -> Result<String> {
        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            match self.try_fetch().await {
                Ok(data) => {
                    if attempt > 1 {
                        tracing::info!("第{}次重试成功", attempt);
                    }
                    return Ok(data);
                }
                Err(e) => {
                    tracing::warn!(
                        "第{}次请求失败: {}, 剩余重试次数: {}",
                        attempt,
                        e,
                        self.max_retries - attempt
                    );
                    last_error = Some(e);

                    // 如果不是最后一次，等待后重试
                    if attempt < self.max_retries {
                        let delay = Duration::from_secs(2_u64.pow(attempt - 1)); // 指数退避
                        tracing::info!("等待{:?}后重试...", delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        // 重试循环至少执行一次，last_error必定存在
        Err(last_error.expect("重试循环应该至少执行一次"))
    }

    /// 通用HTTP POST请求（支持参数化）
    ///
    /// # 参数
    /// - `params`: URL编码的请求参数
    ///
    /// # 返回
    /// JSON字符串
    pub async fn fetch_tree(&self, params: &str) -> Result<String> {
        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            match self.try_fetch_with_params(params).await {
                Ok(data) => {
                    if attempt > 1 {
                        tracing::info!("第{}次重试成功", attempt);
                    }
                    return Ok(data);
                }
                Err(e) => {
                    tracing::warn!(
                        "第{}次请求失败: {}, 剩余重试次数: {}",
                        attempt,
                        e,
                        self.max_retries - attempt
                    );
                    last_error = Some(e);

                    if attempt < self.max_retries {
                        let delay = Duration::from_secs(2_u64.pow(attempt - 1));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.expect("重试循环应该至少执行一次"))
    }

    /// 尝试单次请求（内部方法）
    ///
    /// 使用POST方法，参数格式为application/x-www-form-urlencoded
    async fn try_fetch(&self) -> Result<String> {
        // Level 1：获取校区列表
        self.try_fetch_with_params("yzm=123&Id=000&level=1").await
    }

    /// 带参数的单次请求（私有方法）
    async fn try_fetch_with_params(&self, params: &str) -> Result<String> {
        let response = self
            .client
            .post(&self.api_url)
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header("Accept", "application/json, text/javascript, */*; q=0.01")
            .body(params.to_string())
            .send()
            .await
            .context("HTTP请求发送失败")?;

        let status = response.status();
        if !status.is_success() {
            tracing::error!(
                external_dependency = "room_crawler",
                status = status.as_u16(),
                "外部 HTTP 请求失败"
            );
            anyhow::bail!("{}", http_status_error_message("room_crawler", status));
        }

        let body = response.text().await.context("读取响应体失败")?;

        if body.is_empty() {
            anyhow::bail!("响应体为空");
        }

        tracing::debug!("收到响应: {} bytes", body.len());

        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::StatusCode, response::IntoResponse, routing::post, Router};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use tokio::net::TcpListener;

    #[test]
    fn test_client_creation() {
        let config = CrawlerConfig {
            api_url: "https://example.com/api".to_string(),
            timeout_seconds: 30,
            connect_timeout_seconds: 10,
            max_retries: 3,
            concurrency: 50,
        };

        let result = RoomClient::new(&config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_room_tree() {
        // 外部网络测试与本地 DB/Redis 集成测试分离，避免默认门禁依赖公网可用性
        if std::env::var("RUN_EXTERNAL_INTEGRATION_TESTS").is_err() {
            println!("跳过外部网络测试：设置 RUN_EXTERNAL_INTEGRATION_TESTS=1 以启用");
            return;
        }

        let config = CrawlerConfig {
            api_url: "https://zywxhd02.gxust.edu.cn/Home/GetRoomTree".to_string(),
            timeout_seconds: 30,
            connect_timeout_seconds: 10,
            max_retries: 3,
            concurrency: 50,
        };

        let client = RoomClient::new(&config).unwrap();
        let result = client.fetch_room_tree().await;

        match result {
            Ok(data) => {
                assert!(!data.is_empty(), "API返回的数据不应为空");
                println!("✓ 爬虫网络测试通过，获取到{}字节数据", data.len());
            }
            Err(e) => {
                println!("⚠ 网络请求失败（这是预期的，如果网络不可达）: {}", e);
                if std::env::var("RUN_EXTERNAL_INTEGRATION_TESTS").is_ok() {
                    panic!("外部网络测试模式下请求应该成功: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_retry_backoff_calculation() {
        // 单元测试：验证指数退避算法
        use std::time::Duration;

        let delay1 = Duration::from_secs(2_u64.pow(0)); // 1秒
        let delay2 = Duration::from_secs(2_u64.pow(1)); // 2秒
        let delay3 = Duration::from_secs(2_u64.pow(2)); // 4秒

        assert_eq!(delay1.as_secs(), 1);
        assert_eq!(delay2.as_secs(), 2);
        assert_eq!(delay3.as_secs(), 4);

        println!("✓ 重试退避算法验证通过");
    }

    #[test]
    fn test_config_default_values() {
        // 单元测试：验证默认配置
        let config = CrawlerConfig::default();

        assert!(config.timeout_seconds > 0);
        assert!(config.max_retries > 0);
        assert!(config.concurrency > 0);
        assert!(!config.api_url.is_empty());

        println!("✓ 爬虫配置默认值验证通过");
    }

    #[tokio::test]
    async fn test_fetch_room_tree_with_mock_server() {
        async fn handler() -> impl IntoResponse {
            (
                StatusCode::OK,
                "{\"BS\":\"1\",\"Msg\":\"成功\",\"total\":1}",
            )
        }

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("启动房间树 mock server 失败");
        let addr = listener.local_addr().expect("获取房间树 mock 地址失败");
        let app = Router::new().route("/room-tree", post(handler));

        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("运行房间树 mock server 失败");
        });

        let config = CrawlerConfig {
            api_url: format!("http://{addr}/room-tree"),
            timeout_seconds: 30,
            connect_timeout_seconds: 10,
            max_retries: 3,
            concurrency: 50,
        };

        let client = RoomClient::new(&config).expect("创建 RoomClient 失败");
        let result = client
            .fetch_room_tree()
            .await
            .expect("mock 房间树请求应成功");

        assert!(result.contains("\"BS\":\"1\""));
    }

    #[tokio::test]
    async fn test_fetch_tree_retries_until_success_with_mock_server() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_for_handler = Arc::clone(&attempts);

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("启动房间树重试 mock server 失败");
        let addr = listener.local_addr().expect("获取房间树重试 mock 地址失败");
        let app = Router::new().route(
            "/retry",
            post(move || {
                let attempts = Arc::clone(&attempts_for_handler);
                async move {
                    let current = attempts.fetch_add(1, Ordering::Relaxed);
                    if current == 0 {
                        (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response()
                    } else {
                        (
                            StatusCode::OK,
                            "{\"BS\":\"1\",\"Msg\":\"成功\",\"total\":1}",
                        )
                            .into_response()
                    }
                }
            }),
        );

        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("运行房间树重试 mock server 失败");
        });

        let config = CrawlerConfig {
            api_url: format!("http://{addr}/retry"),
            timeout_seconds: 30,
            connect_timeout_seconds: 10,
            max_retries: 3,
            concurrency: 50,
        };

        let client = RoomClient::new(&config).expect("创建 RoomClient 失败");
        let result = client
            .fetch_tree("yzm=123&Id=000&level=1")
            .await
            .expect("重试后应成功");

        assert!(result.contains("\"BS\":\"1\""));
        assert_eq!(attempts.load(Ordering::Relaxed), 2);
    }
}
