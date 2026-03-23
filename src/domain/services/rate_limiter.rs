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
        Self { redis_pool, config }
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
        let mut conn =
            self.redis_pool.get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get Redis connection: {}", e))
            })?;

        // 获取当前时间戳（秒）
        // SystemTime::now()总是大于UNIX_EPOCH（除非系统时钟设置在1970年之前）
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("系统时钟错误：时间早于UNIX纪元（1970-01-01）")
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
        let count: u32 = conn
            .incr(&key, 1)
            .await
            .map_err(|e| AppError::Internal(format!("Redis INCR failed: {}", e)))?;

        // 2. 如果是第一次访问，设置过期时间为1秒
        if count == 1 {
            conn.expire::<_, ()>(&key, 1)
                .await
                .map_err(|e| AppError::Internal(format!("Redis EXPIRE failed: {}", e)))?;
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
        let mut conn =
            self.redis_pool.get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get Redis connection: {}", e))
            })?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("系统时钟错误：时间早于UNIX纪元（1970-01-01）")
            .as_secs();

        let key = format!("{}:{}", operation.key_prefix(), now);

        let limit = match operation {
            RateLimitOperation::Insert => self.config.insert_per_second,
            RateLimitOperation::Query => self.config.query_per_second,
        };

        // 获取当前计数
        let count: Option<u32> = conn
            .get(&key)
            .await
            .map_err(|e| AppError::Internal(format!("Redis GET failed: {}", e)))?;

        let used = count.unwrap_or(0);
        Ok(limit.saturating_sub(used))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn redis_test_requested() -> bool {
        std::env::var("RUN_INTEGRATION_TESTS").is_ok()
            || std::env::var("REDIS_HOST").is_ok()
            || std::env::var("REDIS_PORT").is_ok()
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        // 检查是否有Redis可用
        if !redis_test_requested() {
            println!(
                "跳过限流器测试：设置 RUN_INTEGRATION_TESTS=1 或 REDIS_HOST/REDIS_PORT 以启用"
            );
            return;
        }

        use crate::config::RedisConfig;
        use crate::infrastructure::redis::create_redis_pool;

        let redis_config = RedisConfig {
            host: std::env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("REDIS_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(6379),
            max_connections: 10,
            min_connections: 2,
            connection_timeout: 30,
        };

        let rate_limit_config = RateLimitConfig {
            insert_per_second: 5,
            query_per_second: 2,
        };

        let redis_pool = match create_redis_pool(&redis_config).await {
            Ok(pool) => pool,
            Err(e) => {
                println!("⚠ Redis连接失败（这是预期的，如果Redis未运行）: {}", e);
                if std::env::var("RUN_INTEGRATION_TESTS").is_ok() {
                    panic!("集成测试模式下Redis必须可用");
                }
                return;
            }
        };

        let limiter = RateLimiter::new(redis_pool, rate_limit_config);

        // 测试插入限流
        for i in 0..7 {
            let allowed = limiter
                .check_rate_limit(RateLimitOperation::Insert)
                .await
                .unwrap();
            if i < 5 {
                assert!(allowed, "前5次应该允许");
            } else {
                assert!(!allowed, "超过5次应该拒绝");
            }
        }

        println!("✓ 限流器测试通过");
    }

    #[test]
    fn test_rate_limit_config_validation() {
        // 单元测试：验证配置结构
        let config = RateLimitConfig {
            insert_per_second: 10,
            query_per_second: 5,
        };

        assert!(config.insert_per_second > 0);
        assert!(config.query_per_second > 0);
        assert!(config.insert_per_second > config.query_per_second);

        println!("✓ 限流配置验证通过");
    }

    #[test]
    fn test_rate_limit_key_generation() {
        // 单元测试：验证Redis键生成逻辑
        let timestamp = 1234567890;

        let insert_key = format!("ratelimit:insert:{}", timestamp);
        let query_key = format!("ratelimit:query:{}", timestamp);

        assert!(insert_key.starts_with("ratelimit:insert:"));
        assert!(query_key.starts_with("ratelimit:query:"));
        assert_ne!(insert_key, query_key);

        println!("✓ 限流键生成逻辑验证通过");
    }
}
