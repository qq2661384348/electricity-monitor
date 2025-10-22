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
        .map_err(|e| {
            AppError::Config(format!("Failed to create database pool: {}", e))
        })?;

    // 测试连接
    let _conn = pool.get().await.map_err(|e| {
        AppError::Config(format!("Failed to get database connection: {}", e))
    })?;

    tracing::info!("Database pool created successfully");

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要实际数据库连接
    async fn test_create_pool() {
        use crate::config::database::DatabaseType;

        let config = DatabaseConfig {
            db_type: DatabaseType::Postgres,
            host: "localhost".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            database: "test".to_string(),
            max_connections: 5,
            min_connections: 1,
            connection_timeout: 30,
        };

        let result = create_pool(&config).await;
        assert!(result.is_ok());
    }
}
