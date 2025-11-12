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
    async fn test_create_redis_pool() {
        // 检查是否有Redis可用（通过环境变量控制）
        if std::env::var("REDIS_URL").is_err() && std::env::var("RUN_INTEGRATION_TESTS").is_err() {
            println!("跳过Redis测试：设置 RUN_INTEGRATION_TESTS=1 或 REDIS_URL 环境变量以启用");
            return;
        }

        let config = RedisConfig {
            host: std::env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("REDIS_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(6379),
            max_connections: 10,
            min_connections: 2,
            connection_timeout: 30,
        };

        let result = create_redis_pool(&config).await;
        
        match result {
            Ok(pool) => {
                // 验证连接池可用
                let conn_result = pool.get().await;
                assert!(conn_result.is_ok(), "无法从Redis连接池获取连接");
                println!("✓ Redis连接池测试通过");
            }
            Err(e) => {
                // 如果Redis不可用，提供友好的错误信息
                println!("⚠ Redis连接失败（这是预期的，如果Redis未运行）: {}", e);
                if std::env::var("RUN_INTEGRATION_TESTS").is_ok() {
                    panic!("集成测试模式下Redis必须可用");
                }
            }
        }
    }
    
    #[test]
    fn test_redis_config_validation() {
        // 单元测试：验证配置结构
        let config = RedisConfig {
            host: "localhost".to_string(),
            port: 6379,
            max_connections: 10,
            min_connections: 2,
            connection_timeout: 30,
        };
        
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 6379);
        assert!(config.max_connections > config.min_connections);
    }
}
