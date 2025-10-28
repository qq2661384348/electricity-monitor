//! 电费插入服务
//! 
//! 后台任务：从Redis队列消费电费数据并插入数据库
//! 应用限流防止过多并发影响主业务

use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::domain::services::{RateLimitOperation, RateLimiter};
use crate::errors::Result;
use crate::infrastructure::repositories::RoomRepository;
use crate::infrastructure::RedisPool;
use deadpool_redis::redis::AsyncCommands;

/// 电费数据结构（从Redis队列消费）
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ElectricityData {
    /// 房间ID
    pub roomid: i32,
    
    /// 电费值
    pub electricity_fee: f32,
}

/// 电费插入服务
pub struct ElectricityService {
    /// Room仓储
    repository: RoomRepository,
    
    /// Redis连接池
    redis_pool: RedisPool,
    
    /// 限流器
    rate_limiter: Arc<RateLimiter>,
    
    /// Redis队列键名
    queue_key: String,
}

impl ElectricityService {
    /// 创建新的电费插入服务
    /// 
    /// # 参数
    /// - `repository`: Room仓储
    /// - `redis_pool`: Redis连接池
    /// - `rate_limiter`: 限流器
    pub fn new(
        repository: RoomRepository,
        redis_pool: RedisPool,
        rate_limiter: Arc<RateLimiter>,
    ) -> Self {
        Self {
            repository,
            redis_pool,
            rate_limiter,
            queue_key: "electricity:insert_queue".to_string(),
        }
    }

    /// 启动后台任务
    /// 
    /// # 说明
    /// 此方法会启动一个Tokio任务，持续从Redis队列消费电费数据
    /// 应用限流（每秒10次）防止过多并发
    pub fn spawn_worker(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            tracing::info!("电费插入服务已启动");
            
            loop {
                // 应用限流
                if let Err(e) = self.rate_limiter
                    .wait_for_rate_limit(RateLimitOperation::Insert)
                    .await
                {
                    tracing::error!("限流检查失败: {}", e);
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }

                // 从Redis队列消费数据
                match self.consume_from_queue().await {
                    Ok(Some(data)) => {
                        // 处理电费数据
                        if let Err(e) = self.process_electricity_data(data).await {
                            tracing::error!("处理电费数据失败: {}", e);
                        }
                    }
                    Ok(None) => {
                        // 队列为空，等待一段时间
                        sleep(Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        tracing::error!("从队列消费数据失败: {}", e);
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        })
    }

    /// 从Redis队列消费数据
    /// 
    /// # 返回
    /// - `Ok(Some(data))`: 成功消费数据
    /// - `Ok(None)`: 队列为空
    /// - `Err`: 消费失败
    async fn consume_from_queue(&self) -> Result<Option<ElectricityData>> {
        let mut conn = self.redis_pool.get().await.map_err(|e| {
            crate::errors::AppError::Internal(format!("Failed to get Redis connection: {}", e))
        })?;

        // 使用BLPOP阻塞式弹出（超时5秒）
        let result: Option<(String, String)> = conn
            .blpop(&self.queue_key, 5.0)
            .await
            .map_err(|e| {
                crate::errors::AppError::Internal(format!("Redis BLPOP failed: {}", e))
            })?;

        match result {
            Some((_key, json_data)) => {
                // 反序列化JSON数据
                let data: ElectricityData = serde_json::from_str(&json_data)
                    .map_err(|e| {
                        crate::errors::AppError::Internal(format!("JSON反序列化失败: {}", e))
                    })?;
                
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    /// 处理电费数据
    /// 
    /// # 参数
    /// - `data`: 电费数据
    /// 
    /// # 说明
    /// UPDATE覆盖旧值，触发器会自动检查阈值并更新send_flag
    async fn process_electricity_data(&self, data: ElectricityData) -> Result<()> {
        tracing::debug!(
            "处理电费数据: roomid={}, electricity_fee={}",
            data.roomid,
            data.electricity_fee
        );

        // 更新数据库（UPDATE覆盖）
        let affected_rows = self
            .repository
            .update_electricity_fee_by_roomid(data.roomid, data.electricity_fee)
            .await?;

        if affected_rows > 0 {
            tracing::info!(
                "成功更新房间电费: roomid={}, electricity_fee={}, affected_rows={}",
                data.roomid,
                data.electricity_fee,
                affected_rows
            );
        } else {
            tracing::warn!(
                "未找到匹配的房间: roomid={}",
                data.roomid
            );
        }

        Ok(())
    }

    /// 推送电费数据到队列（供外部API调用）
    /// 
    /// # 参数
    /// - `data`: 电费数据
    /// 
    /// # 说明
    /// 此方法用于外部模块（如API获取模块）推送电费数据到队列
    pub async fn push_to_queue(&self, data: ElectricityData) -> Result<()> {
        let mut conn = self.redis_pool.get().await.map_err(|e| {
            crate::errors::AppError::Internal(format!("Failed to get Redis connection: {}", e))
        })?;

        // 序列化为JSON
        let json_data = serde_json::to_string(&data)
            .map_err(|e| {
                crate::errors::AppError::Internal(format!("JSON序列化失败: {}", e))
            })?;

        // 推送到队列
        conn.rpush::<_, _, ()>(&self.queue_key, json_data)
            .await
            .map_err(|e| {
                crate::errors::AppError::Internal(format!("Redis RPUSH failed: {}", e))
            })?;

        tracing::debug!("电费数据已推送到队列: {:?}", data);

        Ok(())
    }

    /// 获取队列长度
    pub async fn get_queue_length(&self) -> Result<usize> {
        let mut conn = self.redis_pool.get().await.map_err(|e| {
            crate::errors::AppError::Internal(format!("Failed to get Redis connection: {}", e))
        })?;

        let length: usize = conn
            .llen(&self.queue_key)
            .await
            .map_err(|e| {
                crate::errors::AppError::Internal(format!("Redis LLEN failed: {}", e))
            })?;

        Ok(length)
    }
}

/// 电费获取模块trait（预留接口）
/// 
/// # 说明
/// 外部模块需要实现此trait以提供电费数据
#[allow(dead_code, async_fn_in_trait)]
pub trait ElectricityFetcher: Send + Sync {
    /// 获取房间电费
    /// 
    /// # 参数
    /// - `roomid`: 房间ID
    /// 
    /// # 返回
    /// 电费值
    async fn fetch_electricity_fee(&self, roomid: i32) -> Result<f32>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_electricity_data_serialization() {
        let data = ElectricityData {
            roomid: 101,
            electricity_fee: 123.45,
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: ElectricityData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.roomid, 101);
        assert_eq!(deserialized.electricity_fee, 123.45);
    }
}
