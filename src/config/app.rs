//! 应用程序配置加载和管理

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::sync::OnceLock;

use super::DatabaseConfig;

/// 服务器配置
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// 监听地址
    pub host: String,
    
    /// 监听端口
    pub port: u16,
}

/// JWT配置
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// JWT密钥
    pub secret: String,
    
    /// 过期时间（小时）
    pub expiration_hours: u64,
}

/// 日志配置
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    
    /// 日志格式 (json/pretty)
    pub format: String,
}

/// 应用程序配置
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// 服务器配置
    pub server: ServerConfig,
    
    /// 数据库配置
    pub database: DatabaseConfig,
    
    /// JWT配置
    pub jwt: JwtConfig,
    
    /// 日志配置
    pub logging: LoggingConfig,
}

/// 全局配置单例
static CONFIG: OnceLock<AppConfig> = OnceLock::new();

impl AppConfig {
    /// 加载配置
    pub fn load() -> Result<Self, ConfigError> {
        // 获取运行环境 (development/production)
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        
        tracing::info!("Loading configuration for environment: {}", env);

        let config = Config::builder()
            // 1. 加载默认配置
            .add_source(File::with_name("config/default"))
            // 2. 根据环境加载对应配置（覆盖默认配置）
            .add_source(File::with_name(&format!("config/{}", env)).required(false))
            // 3. 从环境变量加载（最高优先级，前缀为APP_）
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;

        config.try_deserialize()
    }

    /// 获取全局配置实例
    pub fn global() -> &'static AppConfig {
        CONFIG.get().expect("配置未初始化，请先调用 AppConfig::init()")
    }

    /// 初始化全局配置
    pub fn init() -> Result<(), ConfigError> {
        let config = Self::load()?;
        CONFIG.set(config).map_err(|_| {
            ConfigError::Message("配置已经初始化".to_string())
        })?;
        Ok(())
    }

    /// 获取服务器监听地址
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::database::DatabaseType;

    #[test]
    fn test_server_addr() {
        let config = AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8000,
            },
            database: DatabaseConfig {
                db_type: DatabaseType::Postgres,
                host: "localhost".to_string(),
                port: 5432,
                username: "test".to_string(),
                password: "test".to_string(),
                database: "test".to_string(),
                max_connections: 10,
                min_connections: 2,
                connection_timeout: 30,
            },
            jwt: JwtConfig {
                secret: "test-secret".to_string(),
                expiration_hours: 24,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
        };

        assert_eq!(config.server_addr(), "127.0.0.1:8000");
    }
}
