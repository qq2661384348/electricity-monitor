//! 应用程序配置加载和管理

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::sync::OnceLock;

use super::{
    AdminConfig, DatabaseConfig, ElectricityFetcherConfig, NotificationConfig, QQBotConfig,
    RateLimitConfig, RedisConfig, RoomSyncConfig, StaticFilesConfig, VerificationConfig,
};

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
    
    /// 管理员固定Token（配置文件中定义，永久有效）
    pub admin_token: String,
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
    
    /// Redis配置
    pub redis: RedisConfig,
    
    /// 限流配置
    pub rate_limit: RateLimitConfig,
    
    /// 房间同步服务配置
    #[serde(default)]
    pub room_sync: RoomSyncConfig,
    
    /// 电费获取服务配置
    #[serde(default)]
    pub electricity_fetcher: ElectricityFetcherConfig,
    
    /// QQ机器人配置
    #[serde(default)]
    pub qq_bot: QQBotConfig,
    
    /// 验证码配置
    #[serde(default)]
    pub verification: VerificationConfig,
    
    /// 通知配置
    #[serde(default)]
    pub notification: NotificationConfig,
    
    /// 管理员配置
    #[serde(default)]
    pub admin: AdminConfig,
    
    /// 静态文件服务配置
    #[serde(default)]
    pub static_files: StaticFilesConfig,
}

/// 全局配置单例
static CONFIG: OnceLock<AppConfig> = OnceLock::new();

impl AppConfig {
    /// 加载配置
    pub fn load() -> Result<Self, ConfigError> {
        let config = Config::builder()
            // 1. 加载 default.toml（全局唯一配置）
            .add_source(File::with_name("config/default"))
            // 2. 从环境变量加载（最高优先级，前缀为APP_）
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
        
        // 验证电费获取服务配置
        if config.electricity_fetcher.enabled {
            if let Err(e) = config.electricity_fetcher.validate() {
                return Err(ConfigError::Message(format!(
                    "电费获取服务配置验证失败: {}",
                    e
                )));
            }
        }
        
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
                admin_token: "test-admin-token-12345".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
            redis: RedisConfig {
                host: "127.0.0.1".to_string(),
                port: 6379,
                max_connections: 10,
                min_connections: 2,
                connection_timeout: 30,
            },
            rate_limit: RateLimitConfig {
                insert_per_second: 10,
                query_per_second: 1,
            },
            room_sync: RoomSyncConfig::default(),
            electricity_fetcher: ElectricityFetcherConfig::default(),
            qq_bot: QQBotConfig::default(),
            verification: VerificationConfig::default(),
            notification: NotificationConfig::default(),
            admin: AdminConfig::default(),
            static_files: StaticFilesConfig::default(),
        };

        assert_eq!(config.server_addr(), "127.0.0.1:8000");
    }
}
