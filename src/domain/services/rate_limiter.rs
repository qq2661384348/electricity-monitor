//! 限流服务
//! 
//! 使用Redis实现固定窗口限流算法
//! 仅限制电费插入子线程和send_flag查询子线程

use crate::config::RateLimitConfig;
use crate::errors::{AppError, Result};
use crate::infrastructure::RedisPool;
use deadpool_redis::redis::AsyncCommands;
use std::time::{SystemTime, UNIX_EPOCH};

/// 限流操作类型
#[derive(Debug, Clone, Copy)]
pub enum RateLimitOperation {
    /// 电费插入操作
    Insert,
    /// send_flag查询操作
    Query,
}

impl RateLimitOperation {
    /// 获取Redis键前缀
    fn key_prefix(&self) -> &'static str {
        match self {
            Self::Insert => "ratelimit:insert",
            Self::Query => "ratelimit:query",
        }
    }
}

/// 限流服务
#[derive(Clone)]
pub struct RateLimiter {
    redis_pool: RedisPool,
    config: RateLimitConfig,
}

impl RateLimiter {
    /// 创建限流服务实例
    pub fn new(redis_pool: RedisPool, config: RateLimitConfig) -> Self {
        Self {
            redis_pool,
            config,
        }
    }

    /// 检查是否允许执行操作（固定窗口算法）
    /// 
    /// # 参数
    /// - `operation`: 操作类型（插入或查询）
    /// 
    /// # 返回
    /// - `Ok(true)`: 允许执行
    /// - `Ok(false)`: 超过限制，需要等待
    /// - `Err`: Redis错误
    pub async fn check_rate_limit(&self, operation: RateLimitOperation) -> Result<bool> {
        let mut conn = self.redis_pool.get().await.map_err(|e| {
            AppError::Internal(format!("Failed to get Redis connection: {}", e))
        })?;

        // 获取当前时间戳（秒）
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 构造Redis键：ratelimit:insert:1730000000 或 ratelimit:query:1730000000
        let key = format!("{}:{}", operation.key_prefix(), now);

        // 获取限制值
        let limit = match operation {
            RateLimitOperation::Insert => self.config.insert_per_second,
            RateLimitOperation::Query => self.config.query_per_second,
        };

        // 原子操作：INCR + EXPIRE
        // 1. 增加计数器
        let count: u32 = conn.incr(&key, 1).await.map_err(|e| {
            AppError::Internal(format!("Redis INCR failed: {}", e))
        })?;

        // 2. 如果是第一次访问，设置过期时间为1秒
        if count == 1 {
            conn.expire::<_, ()>(&key, 1).await.map_err(|e| {
                AppError::Internal(format!("Redis EXPIRE failed: {}", e))
            })?;
        }

        // 3. 检查是否超过限制
        Ok(count <= limit)
    }

    /// 等待直到允许执行操作
    /// 
    /// # 参数
    /// - `operation`: 操作类型
    /// 
    /// # 注意
    /// 此方法会阻塞当前任务，直到限流允许执行
    pub async fn wait_for_rate_limit(&self, operation: RateLimitOperation) -> Result<()> {
        loop {
            if self.check_rate_limit(operation).await? {
                return Ok(());
            }

            // 等待100ms后重试
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// 获取当前窗口剩余配额
    /// 
    /// # 返回
    /// - 剩余可用次数
    pub async fn get_remaining_quota(&self, operation: RateLimitOperation) -> Result<u32> {
        let mut conn = self.redis_pool.get().await.map_err(|e| {
            AppError::Internal(format!("Failed to get Redis connection: {}", e))
        })?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let key = format!("{}:{}", operation.key_prefix(), now);

        let limit = match operation {
            RateLimitOperation::Insert => self.config.insert_per_second,
            RateLimitOperation::Query => self.config.query_per_second,
        };

        // 获取当前计数
        let count: Option<u32> = conn.get(&key).await.map_err(|e| {
            AppError::Internal(format!("Redis GET failed: {}", e))
        })?;

        let used = count.unwrap_or(0);
        Ok(limit.saturating_sub(used))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要实际Redis连接
    async fn test_rate_limiter() {
        use crate::infrastructure::redis::create_redis_pool;
        use crate::config::RedisConfig;

        let redis_config = RedisConfig {
            host: "127.0.0.1".to_string(),
            port: 6379,
            max_connections: 10,
            min_connections: 2,
            connection_timeout: 30,
        };

        let rate_limit_config = RateLimitConfig {
            insert_per_second: 5,
            query_per_second: 2,
        };

        let redis_pool = create_redis_pool(&redis_config).await.unwrap();
        let limiter = RateLimiter::new(redis_pool, rate_limit_config);

        // 测试插入限流
        for i in 0..7 {
            let allowed = limiter.check_rate_limit(RateLimitOperation::Insert).await.unwrap();
            if i < 5 {
                assert!(allowed, "前5次应该允许");
            } else {
                assert!(!allowed, "超过5次应该拒绝");
            }
        }
    }
}
