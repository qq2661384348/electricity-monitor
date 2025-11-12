//! 批量电费获取执行器
//!
//! 高并发批量查询，使用 Semaphore 限流 + Arc 共享

use super::{http_client::ReqwestAsyncClient, parser::ElectricityParser};
use futures_util::stream::{self, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// 房间查询结果
#[derive(Debug, Clone)]
pub struct RoomResult {
    /// 房间 ID（i32类型）
    pub room_id: i32,
    /// 电费值（成功时为 Some）
    pub electricity: Option<f32>,
    /// 错误信息（失败时为 Some）
    pub error: Option<String>,
}

/// 批量电费获取器
///
/// 高并发批量查询，Semaphore 限流 + Arc 共享
pub struct RoomBatchFetcher {
    url_template: Arc<String>,
    http_client: Arc<ReqwestAsyncClient>,
    parser: Arc<ElectricityParser>,
    semaphore: Arc<Semaphore>,
}

impl RoomBatchFetcher {
    /// 创建批量获取器
    ///
    /// # 参数
    /// - `url_template`: URL模板，例如 "https://api.example.com/electricity?roomid="
    /// - `max_concurrent`: 最大并发数（推荐50）
    pub fn new(url_template: String, max_concurrent: usize) -> super::error::Result<Self> {
        // 验证URL模板
        if !url_template.contains("?roomid=") && !url_template.ends_with("?roomid=") {
            return Err(super::error::ElectricityFetchError::InvalidUrlPrefix(
                "URL模板必须包含 ?roomid=".to_string(),
            ));
        }

        let http_client = ReqwestAsyncClient::new(true)?;
        let parser = ElectricityParser::new()?;

        Ok(Self {
            url_template: Arc::new(url_template),
            http_client: Arc::new(http_client),
            parser: Arc::new(parser),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        })
    }

    /// 获取单个房间电费
    async fn fetch_one(&self, room_id: i32) -> RoomResult {
        // 获取许可（限流）
        let _permit = self.semaphore.acquire().await.unwrap();

        // 构建 URL（i32转字符串）
        let url = format!("{}{}", self.url_template, room_id);

        // 执行请求
        match self.http_client.get(&url).await {
            Ok(response) => {
                let electricity = self.parser.parse(&response);
                RoomResult {
                    room_id,
                    electricity,
                    error: None,
                }
            }
            Err(e) => RoomResult {
                room_id,
                electricity: None,
                error: Some(e.to_string()),
            },
        }
    }

    /// 批量获取电费（HashMap模式）
    ///
    /// # 参数
    /// - `room_ids`: 房间ID列表（i32类型）
    ///
    /// # 返回
    /// - 成功的HashMap<i32, f32>
    /// - 失败的列表会在DEBUG日志中输出
    pub async fn fetch_batch(&self, room_ids: Vec<i32>) -> HashMap<i32, f32> {
        let tasks: Vec<_> = room_ids
            .into_iter()
            .map(|room_id| {
                let fetcher = self.clone_inner();
                tokio::spawn(async move { fetcher.fetch_one(room_id).await })
            })
            .collect();

        let mut results = HashMap::new();
        for task in tasks {
            if let Ok(result) = task.await {
                if let Some(fee) = result.electricity {
                    results.insert(result.room_id, fee);
                } else {
                    // 失败房间DEBUG日志
                    tracing::debug!(
                        roomid = result.room_id,
                        error = ?result.error,
                        "电费获取失败"
                    );
                }
            }
        }

        results
    }

    /// 流式处理（推荐，内存友好）
    ///
    /// 返回异步 Stream，可以逐个处理结果
    pub async fn fetch_stream(
        &self,
        room_ids: Vec<i32>,
    ) -> impl futures_util::Stream<Item = RoomResult> + '_ {
        let max_concurrent = self.semaphore.available_permits();

        stream::iter(room_ids)
            .map(move |room_id| {
                let fetcher = self.clone_inner();
                async move { fetcher.fetch_one(room_id).await }
            })
            .buffer_unordered(max_concurrent)
    }

    /// 内部克隆（Arc 引用计数）
    fn clone_inner(&self) -> Self {
        Self {
            url_template: Arc::clone(&self.url_template),
            http_client: Arc::clone(&self.http_client),
            parser: Arc::clone(&self.parser),
            semaphore: Arc::clone(&self.semaphore),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetcher_creation() {
        let fetcher = RoomBatchFetcher::new(
            "https://example.com/api?roomid=".to_string(),
            50,
        );
        assert!(fetcher.is_ok());
    }

    #[test]
    fn test_invalid_url_template() {
        let fetcher = RoomBatchFetcher::new("https://example.com/api".to_string(), 50);
        assert!(fetcher.is_err());
    }
}
