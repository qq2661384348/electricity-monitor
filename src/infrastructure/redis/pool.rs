//! Redis连接池管理

use deadpool_redis::{redis::cmd, Config, Pool, Runtime};

use crate::config::RedisConfig;
use crate::errors::{AppError, Result};

/// Redis连接池类型
pub type RedisPool = Pool;

/// 创建Redis连接池
pub async fn create_redis_pool(config: &RedisConfig) -> Result<RedisPool> {
    let redis_url = config.connection_url();
    
    tracing::info!(
        "Creating Redis pool: host={}, port={}",
        config.host,
        config.port
    );

    // 创建连接池配置
    let pool_config = Config::from_url(redis_url);
    
    // 构建连接池
    let pool = pool_config
        .create_pool(Some(Runtime::Tokio1))
        .map_err(|e| {
            AppError::Config(format!("Failed to create Redis pool: {}", e))
        })?;

    // 测试连接
    let mut conn = pool.get().await.map_err(|e| {
        AppError::Config(format!("Failed to get Redis connection: {}", e))
    })?;
    
    // 执行PING命令测试连接
    cmd("PING")
        .query_async::<()>(&mut conn)
        .await
        .map_err(|e| {
            AppError::Config(format!("Redis PING failed: {}", e))
        })?;

    tracing::info!("Redis pool created successfully");

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要实际Redis连接
    async fn test_create_redis_pool() {
        let config = RedisConfig {
            host: "127.0.0.1".to_string(),
            port: 6379,
            max_connections: 10,
            min_connections: 2,
            connection_timeout: 30,
        };

        let result = create_redis_pool(&config).await;
        assert!(result.is_ok());
    }
}
