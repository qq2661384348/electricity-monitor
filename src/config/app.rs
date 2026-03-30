//! 应用程序配置加载和管理

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::sync::OnceLock;

use super::{
    AdminConfig, DatabaseConfig, ElectricityFetcherConfig, NotificationConfig, QQBotConfig,
    RateLimitConfig, RedisConfig, RoomSyncConfig, StaticFilesConfig, VerificationConfig,
};

const DEFAULT_ENVIRONMENT: &str = "development";

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

    /// JWT secret file 路径
    #[serde(default)]
    pub secret_file: Option<String>,

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
        let environment = Self::current_environment();
        Self::load_for_environment(&environment)
    }

    /// 按指定环境加载配置
    pub fn load_for_environment(environment: &str) -> Result<Self, ConfigError> {
        Self::load_for_environment_with_source(environment, None)
    }

    fn load_for_environment_with_source(
        environment: &str,
        env_source: Option<HashMap<String, String>>,
    ) -> Result<Self, ConfigError> {
        let environment = environment.trim().to_ascii_lowercase();
        let environment = if environment.is_empty() {
            DEFAULT_ENVIRONMENT.to_string()
        } else {
            environment
        };
        let env_config_path = format!("config/{}", environment);
        let mut environment_source = Environment::with_prefix("APP").separator("__");

        if let Some(source) = env_source {
            environment_source = environment_source.source(Some(source));
        }

        let config = Config::builder()
            // 1. 加载 default.toml（全局唯一配置）
            .add_source(File::with_name("config/default"))
            // 2. 加载环境配置（如果存在则覆盖 default.toml）
            .add_source(File::with_name(&env_config_path).required(false))
            // 3. 从环境变量加载（最高优先级，命名规则为 APP__<SECTION>__<KEY>）
            .add_source(environment_source)
            .build()?;

        let app_config = Self::resolve_secrets(config.try_deserialize::<Self>()?, &environment)?;
        Self::validate_environment_rules(&app_config, &environment)?;
        Self::validate_sensitive_config(&app_config, &environment)?;
        Ok(app_config)
    }

    /// 获取当前环境名称
    pub fn current_environment() -> String {
        std::env::var("APP_ENV")
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_ENVIRONMENT.to_string())
    }

    /// 获取全局配置实例
    pub fn global() -> &'static AppConfig {
        CONFIG
            .get()
            .expect("配置未初始化，请先调用 AppConfig::init()")
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

        CONFIG
            .set(config)
            .map_err(|_| ConfigError::Message("配置已经初始化".to_string()))?;
        Ok(())
    }

    /// 获取服务器监听地址
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    fn validate_environment_rules(config: &Self, environment: &str) -> Result<(), ConfigError> {
        if environment == DEFAULT_ENVIRONMENT {
            if !Self::is_local_host(&config.database.host) {
                return Err(ConfigError::Message(format!(
                    "development 环境只允许连接本地数据库，当前 database.host={}。请使用 localhost、127.0.0.1、::1 或 host.docker.internal。",
                    config.database.host
                )));
            }

            if !Self::is_local_host(&config.redis.host) {
                return Err(ConfigError::Message(format!(
                    "development 环境只允许连接本地 Redis，当前 redis.host={}。请使用 localhost、127.0.0.1、::1 或 host.docker.internal。",
                    config.redis.host
                )));
            }
        }

        Ok(())
    }

    fn resolve_secrets(mut config: Self, environment: &str) -> Result<Self, ConfigError> {
        if let Some(secret_file) = config.jwt.secret_file.as_deref() {
            config.jwt.secret = Self::read_secret_file(secret_file, "jwt.secret_file")?;
        }

        if let Some(secret_file) = config.database.password_file.as_deref() {
            config.database.password =
                Self::read_secret_file(secret_file, "database.password_file")?;
        }

        if let Some(secret_file) = config.qq_bot.bearer_token_file.as_deref() {
            config.qq_bot.bearer_token =
                Self::read_secret_file(secret_file, "qq_bot.bearer_token_file")?;
        }

        tracing::info!(
            environment = environment,
            jwt_secret_from_file = config.jwt.secret_file.is_some(),
            database_password_from_file = config.database.password_file.is_some(),
            qq_bot_token_from_file = config.qq_bot.bearer_token_file.is_some(),
            "敏感配置解析完成"
        );

        Ok(config)
    }

    fn validate_sensitive_config(config: &Self, environment: &str) -> Result<(), ConfigError> {
        if environment != "production" {
            return Ok(());
        }

        let missing = [
            (
                "jwt.secret_file",
                config.jwt.secret_file.as_ref(),
                config.jwt.secret.trim(),
            ),
            (
                "database.password_file",
                config.database.password_file.as_ref(),
                config.database.password.trim(),
            ),
            (
                "qq_bot.bearer_token_file",
                config.qq_bot.bearer_token_file.as_ref(),
                config.qq_bot.bearer_token.trim(),
            ),
        ]
        .into_iter()
        .filter_map(|(field, file, value)| {
            if file.is_none() || value.is_empty() || value.starts_with("CHANGE-THIS") {
                Some(field)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

        if missing.is_empty() {
            return Ok(());
        }

        Err(ConfigError::Message(format!(
            "production 环境要求通过 Compose secrets 注入敏感配置，以下字段缺失或未通过 secret file 提供: {}",
            missing.join(", ")
        )))
    }

    fn read_secret_file(path: &str, field_name: &str) -> Result<String, ConfigError> {
        let content = fs::read_to_string(path).map_err(|error| {
            ConfigError::Message(format!(
                "读取 secret file 失败: field={}, path={}, error={}",
                field_name, path, error
            ))
        })?;

        let secret = content.trim().to_string();
        if secret.is_empty() {
            return Err(ConfigError::Message(format!(
                "secret file 为空: field={}, path={}",
                field_name, path
            )));
        }

        Ok(secret)
    }

    fn is_local_host(host: &str) -> bool {
        matches!(
            host.trim().to_ascii_lowercase().as_str(),
            "localhost" | "127.0.0.1" | "::1" | "host.docker.internal"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::database::DatabaseType;
    use std::collections::HashMap;
    use std::fs;

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
                password_file: None,
                database: "test".to_string(),
                max_connections: 10,
                min_connections: 2,
                connection_timeout: 30,
            },
            jwt: JwtConfig {
                secret: "test-secret".to_string(),
                secret_file: None,
                expiration_hours: 24,
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

    #[test]
    fn test_development_environment_requires_local_dependencies() {
        let config = AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8000,
            },
            database: DatabaseConfig {
                db_type: DatabaseType::Postgres,
                host: "db.example.internal".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "secret".to_string(),
                password_file: None,
                database: "electricity_dev".to_string(),
                max_connections: 10,
                min_connections: 2,
                connection_timeout: 30,
            },
            jwt: JwtConfig {
                secret: "test-secret".to_string(),
                secret_file: None,
                expiration_hours: 24,
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

        let err = AppConfig::validate_environment_rules(&config, "development")
            .expect_err("development 环境应拒绝远程数据库");

        assert!(err.to_string().contains("database.host=db.example.internal"));
    }

    #[test]
    fn test_development_environment_allows_local_database_and_redis() {
        let config = AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8000,
            },
            database: DatabaseConfig {
                db_type: DatabaseType::Postgres,
                host: "localhost".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "secret".to_string(),
                password_file: None,
                database: "electricity_dev".to_string(),
                max_connections: 10,
                min_connections: 2,
                connection_timeout: 30,
            },
            jwt: JwtConfig {
                secret: "test-secret".to_string(),
                secret_file: None,
                expiration_hours: 24,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
            redis: RedisConfig {
                host: "host.docker.internal".to_string(),
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

        AppConfig::validate_environment_rules(&config, "development")
            .expect("development 环境应允许本地数据库和 Redis");
    }

    #[test]
    fn test_environment_variables_override_nested_keys_with_double_underscore() {
        let mut env = HashMap::new();
        env.insert(
            "APP__DATABASE__HOST".to_string(),
            "host.docker.internal".to_string(),
        );
        env.insert("APP__REDIS__HOST".to_string(), "::1".to_string());

        let config = AppConfig::load_for_environment_with_source("development", Some(env))
            .expect("双下划线环境变量应能覆盖嵌套配置");

        assert_eq!(config.database.host, "host.docker.internal");
        assert_eq!(config.redis.host, "::1");
    }

    #[test]
    fn test_environment_variables_parse_numeric_and_bool_values() {
        let mut env = HashMap::new();
        env.insert("APP__DATABASE__PORT".to_string(), "15432".to_string());
        env.insert("APP__REDIS__PORT".to_string(), "16379".to_string());
        env.insert(
            "APP__DATABASE__MAX_CONNECTIONS".to_string(),
            "11".to_string(),
        );
        env.insert("APP__JWT__EXPIRATION_HOURS".to_string(), "48".to_string());
        env.insert("APP__ROOM_SYNC__ENABLED".to_string(), "false".to_string());

        let config = AppConfig::load_for_environment_with_source("development", Some(env))
            .expect("数值和布尔环境变量应能按目标类型解析");

        assert_eq!(config.database.port, 15432);
        assert_eq!(config.redis.port, 16379);
        assert_eq!(config.database.max_connections, 11);
        assert_eq!(config.jwt.expiration_hours, 48);
        assert!(!config.room_sync.enabled);
    }

    #[test]
    fn test_environment_variables_preserve_numeric_like_string_values() {
        let mut env = HashMap::new();
        env.insert("APP__JWT__SECRET".to_string(), "00123456".to_string());
        env.insert(
            "APP__ADMIN__DEFAULT_QQ_NUMBER".to_string(),
            "00100000001".to_string(),
        );

        let config = AppConfig::load_for_environment_with_source("development", Some(env))
            .expect("字符串类型环境变量不应因 try_parsing 被破坏");

        assert_eq!(config.jwt.secret, "00123456");
        assert_eq!(config.admin.default_qq_number, "00100000001");
    }

    #[test]
    fn test_secret_file_overrides_sensitive_values() {
        let temp_dir = std::env::temp_dir().join(format!(
            "electricity-monitor-config-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&temp_dir).unwrap();
        let jwt_secret_path = temp_dir.join("jwt_secret");
        let db_password_path = temp_dir.join("db_password");
        let qq_token_path = temp_dir.join("qq_token");

        fs::write(&jwt_secret_path, "secret-from-file\n").unwrap();
        fs::write(&db_password_path, "db-password-from-file\n").unwrap();
        fs::write(&qq_token_path, "qq-token-from-file\n").unwrap();

        let mut env = HashMap::new();
        env.insert(
            "APP__JWT__SECRET_FILE".to_string(),
            jwt_secret_path.to_string_lossy().to_string(),
        );
        env.insert(
            "APP__DATABASE__PASSWORD_FILE".to_string(),
            db_password_path.to_string_lossy().to_string(),
        );
        env.insert(
            "APP__QQ_BOT__BEARER_TOKEN_FILE".to_string(),
            qq_token_path.to_string_lossy().to_string(),
        );

        let config = AppConfig::load_for_environment_with_source("development", Some(env))
            .expect("secret file 覆盖应成功");

        assert_eq!(config.jwt.secret, "secret-from-file");
        assert_eq!(config.database.password, "db-password-from-file");
        assert_eq!(config.qq_bot.bearer_token, "qq-token-from-file");

        let _ = fs::remove_file(jwt_secret_path);
        let _ = fs::remove_file(db_password_path);
        let _ = fs::remove_file(qq_token_path);
        let _ = fs::remove_dir(temp_dir);
    }

    #[test]
    fn test_production_requires_secret_files() {
        let err = AppConfig::load_for_environment_with_source("production", Some(HashMap::new()))
            .expect_err("production 环境缺少 secret file 时应失败");

        assert!(err.to_string().contains("Compose secrets"));
    }
}
