//! 数据库连接池管理

use diesel_async::pooled_connection::deadpool::Pool as AsyncPool;
use diesel_async::AsyncPgConnection;

use crate::config::DatabaseConfig;
use crate::errors::{AppError, Result};

/// 数据库连接池类型 (PostgreSQL)
pub type DbPool = AsyncPool<AsyncPgConnection>;

/// 创建数据库连接池
pub async fn create_pool(config: &DatabaseConfig) -> Result<DbPool> {
    let database_url = config.connection_url();

    tracing::info!(
        "Creating database pool: type={:?}, host={}, database={}",
        config.db_type,
        config.host,
        config.database
    );

    // 创建连接池管理器
    let manager = diesel_async::pooled_connection::AsyncDieselConnectionManager::<
        diesel_async::AsyncPgConnection,
    >::new(database_url);

    // 构建连接池
    let pool = AsyncPool::builder(manager)
        .max_size(config.max_connections as usize)
        .build()
        .map_err(|e| AppError::Config(format!("Failed to create database pool: {}", e)))?;

    // 测试连接
    let _conn = pool
        .get()
        .await
        .map_err(|e| AppError::Config(format!("Failed to get database connection: {}", e)))?;

    tracing::info!("Database pool created successfully");

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_test_database_config() -> Option<DatabaseConfig> {
        if std::env::var("RUN_INTEGRATION_TESTS").is_err() {
            println!("跳过数据库测试：设置 RUN_INTEGRATION_TESTS=1 以启用");
            return None;
        }

        let config = crate::config::AppConfig::load_for_environment("development")
            .expect("无法加载 development 配置，数据库集成测试无法继续");

        Some(config.database)
    }

    #[tokio::test]
    async fn test_create_pool() {
        let Some(config) = load_test_database_config() else {
            return;
        };

        let result = create_pool(&config).await;
        assert!(result.is_ok(), "数据库连接池创建失败: {:?}", result.err());

        // 验证连接池可用
        if let Ok(pool) = result {
            let conn_result = pool.get().await;
            assert!(conn_result.is_ok(), "无法从连接池获取连接");
        }
    }
}
