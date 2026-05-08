//! 应用程序配置加载和管理

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::{
    admin::ADMIN_QQ_PLACEHOLDER, auth::AuthConfig, captcha::CaptchaConfig, cors::CorsConfig,
    AdminConfig, DatabaseConfig, ElectricityFetcherConfig, EmailConfig, NotificationConfig,
    PublicSiteConfig, QQBotConfig, RateLimitConfig, RedisConfig, RoomSyncConfig, StaticFilesConfig,
    VerificationConfig,
};

const CONFIG_DIR: &str = "config";
const DEFAULT_ENVIRONMENT: &str = "development";
const SUPPORTED_ENVIRONMENTS: [&str; 2] = ["development", "production"];
const RUNTIME_CONFIG_FILENAMES: [&str; 2] = ["development.toml", "production.toml"];
const DEVELOPMENT_DATABASE_PASSWORD_PLACEHOLDER: &str = "CHANGE-THIS-LOCAL-POSTGRES-PASSWORD";
const CORS_ALLOW_ORIGINS_PLACEHOLDER: &str = "CHANGE-THIS-PRODUCTION-FRONTEND-ORIGIN";
const QQ_BOT_API_URL_PLACEHOLDER: &str = "你的napcat应用URL";
const QQ_BOT_PUBLIC_QQ_PLACEHOLDER: &str = "CHANGE-THIS-QQ-BOT-PUBLIC-QQ";

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

    /// 浏览器认证配置
    #[serde(default)]
    pub auth: AuthConfig,

    /// 日志配置
    pub logging: LoggingConfig,

    /// Redis配置
    pub redis: RedisConfig,

    /// CORS 配置
    #[serde(default)]
    pub cors: CorsConfig,

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

    /// 第三方图形验证码配置
    #[serde(default)]
    pub captcha: CaptchaConfig,

    /// 验证码配置
    #[serde(default)]
    pub verification: VerificationConfig,

    /// 通知配置
    #[serde(default)]
    pub notification: NotificationConfig,

    /// 邮件发送配置
    #[serde(default)]
    pub email: EmailConfig,

    /// 管理员配置
    #[serde(default)]
    pub admin: AdminConfig,

    /// 公开站点访问配置
    #[serde(default)]
    pub public_site: PublicSiteConfig,

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
        Self::load_for_environment_in_dir(environment, env_source, Path::new("."))
    }

    fn load_for_environment_in_dir(
        environment: &str,
        env_source: Option<HashMap<String, String>>,
        base_dir: &Path,
    ) -> Result<Self, ConfigError> {
        let environment = Self::normalize_environment(environment)?;
        Self::resolve_runtime_config_path(base_dir, &environment)?;
        let runtime_config = base_dir.join(CONFIG_DIR).join(&environment);
        let mut environment_source = Environment::with_prefix("APP").separator("__");

        if let Some(source) = env_source {
            environment_source = environment_source.source(Some(source));
        }

        let config = Config::builder()
            .add_source(File::with_name(runtime_config.to_string_lossy().as_ref()))
            .add_source(environment_source)
            .build()?;

        let app_config = Self::resolve_secrets(config.try_deserialize::<Self>()?, &environment)?;
        Self::validate_environment_rules(&app_config, &environment)?;
        Self::validate_public_runtime_config(&app_config)?;
        Self::validate_sensitive_config(&app_config, &environment)?;
        Self::validate_security_contracts(&app_config, &environment)?;
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

            if config.database.password.trim().is_empty()
                || config.database.password == DEVELOPMENT_DATABASE_PASSWORD_PLACEHOLDER
            {
                return Err(ConfigError::Message(format!(
                    "development 环境要求在 {} 中显式设置 database.password。请先从 {} 复制生成运行时配置，再把数据库密码改成当前本地 PostgreSQL 的真实密码。",
                    Self::runtime_config_path(DEFAULT_ENVIRONMENT).display(),
                    Self::suggested_template_path(DEFAULT_ENVIRONMENT).display()
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

        if let Some(secret_file) = config.email.smtp_password_file.as_deref() {
            config.email.smtp_password =
                Self::read_secret_file(secret_file, "email.smtp_password_file")?;
        }

        tracing::info!(
            environment = environment,
            jwt_secret_from_file = config.jwt.secret_file.is_some(),
            database_password_from_file = config.database.password_file.is_some(),
            qq_bot_token_from_file = config.qq_bot.bearer_token_file.is_some(),
            email_password_from_file = config.email.smtp_password_file.is_some(),
            "敏感配置解析完成"
        );

        Ok(config)
    }

    fn validate_sensitive_config(config: &Self, environment: &str) -> Result<(), ConfigError> {
        if environment != "production" {
            return Ok(());
        }

        let mut missing = [
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

        if config.email.requires_secret_file_in_production()
            && (config.email.smtp_password_file.is_none()
                || !config.email.has_valid_resolved_password())
        {
            missing.push("email.smtp_password_file");
        }

        if missing.is_empty() {
            return Ok(());
        }

        Err(ConfigError::Message(format!(
            "production 环境要求通过 Compose secrets 注入敏感配置，以下字段缺失或未通过 secret file 提供: {}",
            missing.join(", ")
        )))
    }

    fn validate_public_runtime_config(config: &Self) -> Result<(), ConfigError> {
        Self::validate_qq_bot_runtime_config(&config.qq_bot)?;
        Self::validate_required_qq_number(
            "qq_bot.public_qq_number",
            &config.qq_bot.public_qq_number,
            QQ_BOT_PUBLIC_QQ_PLACEHOLDER,
        )?;
        Self::validate_required_qq_number(
            "admin.default_qq_number",
            &config.admin.default_qq_number,
            ADMIN_QQ_PLACEHOLDER,
        )?;
        Self::validate_public_site_config(&config.public_site)?;

        if config.captcha.api_url.trim().is_empty()
            || config.captcha.api_url.trim().starts_with("CHANGE-THIS")
        {
            return Err(ConfigError::Message(
                "captcha.api_url 必须配置为真实第三方验证码 API 地址。".to_string(),
            ));
        }

        if config.captcha.request_timeout_seconds == 0 {
            return Err(ConfigError::Message(
                "captcha.request_timeout_seconds 必须大于 0。".to_string(),
            ));
        }

        if config.captcha.token_expire_seconds == 0 {
            return Err(ConfigError::Message(
                "captcha.token_expire_seconds 必须大于 0。".to_string(),
            ));
        }

        let captcha_type = config.captcha.captcha_type.trim().to_ascii_lowercase();
        if !matches!(captcha_type.as_str(), "string" | "math" | "digit") {
            return Err(ConfigError::Message(format!(
                "captcha.captcha_type={} 非法，仅支持 string、math 或 digit。",
                config.captcha.captcha_type
            )));
        }

        if config.captcha.width == 0 || config.captcha.height == 0 {
            return Err(ConfigError::Message(
                "captcha.width 与 captcha.height 必须大于 0。".to_string(),
            ));
        }

        if !(1..=3).contains(&config.captcha.options) {
            return Err(ConfigError::Message(
                "captcha.options 仅支持 1、2 或 3。".to_string(),
            ));
        }

        if config.verification.code_length == 0 || config.verification.code_length > 20 {
            return Err(ConfigError::Message(
                "verification.code_length 必须在 1 到 20 之间。".to_string(),
            ));
        }

        if config.verification.expire_seconds == 0 {
            return Err(ConfigError::Message(
                "verification.expire_seconds 必须大于 0。".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_qq_bot_runtime_config(config: &QQBotConfig) -> Result<(), ConfigError> {
        let api_url = config.api_url.trim();
        if api_url.is_empty()
            || api_url == QQ_BOT_API_URL_PLACEHOLDER
            || api_url.starts_with("CHANGE-THIS")
        {
            return Err(ConfigError::Message(
                "qq_bot.api_url 必须配置为真实 NapCat send_private_msg HTTP 地址，不能留空或使用占位值。".to_string(),
            ));
        }

        if !(api_url.starts_with("http://") || api_url.starts_with("https://"))
            || api_url.chars().any(|ch| ch.is_ascii_whitespace())
        {
            return Err(ConfigError::Message(
                "qq_bot.api_url 必须是以 http:// 或 https:// 开头且不包含空白字符的 HTTP 地址。"
                    .to_string(),
            ));
        }

        let bearer_token = config.bearer_token.trim();
        if bearer_token.is_empty() || bearer_token.starts_with("CHANGE-THIS") {
            return Err(ConfigError::Message(
                "qq_bot.bearer_token 必须通过运行时配置或 secret file 注入真实 token，不能留空或使用占位值。".to_string(),
            ));
        }

        if config.timeout_seconds == 0 {
            return Err(ConfigError::Message(
                "qq_bot.timeout_seconds 必须大于 0。".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_public_site_config(config: &PublicSiteConfig) -> Result<(), ConfigError> {
        let domain = config.domain.trim();
        if domain.is_empty() || domain.starts_with("CHANGE-THIS") {
            return Err(ConfigError::Message(
                "public_site.domain 必须配置为真实公开访问域名，不能留空或使用占位值。".to_string(),
            ));
        }

        if domain.contains("://")
            || domain.contains('/')
            || domain.chars().any(|ch| ch.is_ascii_whitespace())
        {
            return Err(ConfigError::Message(
                "public_site.domain 只填写域名本身，不能包含协议、路径或空白字符。".to_string(),
            ));
        }

        let port = config.port.trim();
        if port.is_empty() || port.starts_with("CHANGE-THIS") {
            return Err(ConfigError::Message(
                "public_site.port 必须配置为真实公开访问端口，不能留空或使用占位值。".to_string(),
            ));
        }

        let parsed_port = port.parse::<u16>().map_err(|_| {
            ConfigError::Message("public_site.port 必须是 1 到 65535 之间的端口号。".to_string())
        })?;
        if parsed_port == 0 {
            return Err(ConfigError::Message(
                "public_site.port 必须是 1 到 65535 之间的端口号。".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_required_qq_number(
        field_name: &str,
        value: &str,
        placeholder: &str,
    ) -> Result<(), ConfigError> {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed == placeholder || trimmed.starts_with("CHANGE-THIS") {
            return Err(ConfigError::Message(format!(
                "{} 必须配置为真实 QQ 号，不能留空或使用占位值。",
                field_name
            )));
        }

        if !(5..=20).contains(&trimmed.len()) || !trimmed.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(ConfigError::Message(format!(
                "{} 必须是 5 到 20 位数字。",
                field_name
            )));
        }

        Ok(())
    }

    fn validate_security_contracts(config: &Self, environment: &str) -> Result<(), ConfigError> {
        let cors_origins = config.cors.origin_list();
        if cors_origins.is_empty() {
            return Err(ConfigError::Message(
                "cors.allowed_origins 不能为空，至少要显式配置一个前端 Origin。".to_string(),
            ));
        }

        let same_site = config
            .auth
            .refresh_cookie_same_site
            .trim()
            .to_ascii_lowercase();
        if !matches!(same_site.as_str(), "lax" | "strict" | "none") {
            return Err(ConfigError::Message(format!(
                "auth.refresh_cookie_same_site={} 非法，仅支持 lax、strict 或 none。",
                config.auth.refresh_cookie_same_site
            )));
        }

        if config.auth.refresh_expiration_hours == 0 {
            return Err(ConfigError::Message(
                "auth.refresh_expiration_hours 必须大于 0。".to_string(),
            ));
        }

        if same_site == "none" && !config.auth.refresh_cookie_secure {
            return Err(ConfigError::Message(
                "auth.refresh_cookie_same_site=none 时必须同时启用 auth.refresh_cookie_secure=true。"
                    .to_string(),
            ));
        }

        if environment != "production" {
            return Ok(());
        }

        if cors_origins.iter().any(|origin| {
            let normalized = origin.trim().to_ascii_lowercase();
            normalized.is_empty()
                || normalized.contains("localhost")
                || normalized.contains("127.0.0.1")
                || normalized.contains(CORS_ALLOW_ORIGINS_PLACEHOLDER.to_ascii_lowercase().as_str())
        }) {
            return Err(ConfigError::Message(
                "production 环境要求 cors.allowed_origins 只包含真实前端 Origin，不能保留 localhost 或占位值。"
                    .to_string(),
            ));
        }

        let admin_qq = config.admin.default_qq_number.trim();
        if admin_qq.is_empty() || admin_qq == ADMIN_QQ_PLACEHOLDER {
            return Err(ConfigError::Message(
                "production 环境要求 admin.default_qq_number 配置为真实管理员 QQ，不能留空或使用占位值。"
                    .to_string(),
            ));
        }

        if !config.auth.refresh_cookie_secure {
            return Err(ConfigError::Message(
                "production 环境要求 auth.refresh_cookie_secure=true。".to_string(),
            ));
        }

        Ok(())
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

    fn normalize_environment(environment: &str) -> Result<String, ConfigError> {
        let normalized = environment.trim().to_ascii_lowercase();
        let normalized = if normalized.is_empty() {
            DEFAULT_ENVIRONMENT.to_string()
        } else {
            normalized
        };

        if SUPPORTED_ENVIRONMENTS.contains(&normalized.as_str()) {
            Ok(normalized)
        } else {
            Err(ConfigError::Message(format!(
                "不支持的 APP_ENV={}。当前仅支持 development 或 production。",
                normalized
            )))
        }
    }

    fn resolve_runtime_config_path(
        base_dir: &Path,
        environment: &str,
    ) -> Result<PathBuf, ConfigError> {
        let config_dir = base_dir.join(CONFIG_DIR);
        let runtime_configs = Self::collect_runtime_configs(&config_dir)?;
        let expected_path = base_dir.join(Self::runtime_config_path(environment));
        let expected_name = Self::runtime_config_filename(environment);

        match runtime_configs.as_slice() {
            [] => Err(ConfigError::Message(format!(
                "缺少运行时配置文件 {}。请先复制 {} 为 {}。config/ 目录下只能保留一个运行时 TOML，且文件名只能是 development.toml 或 production.toml。",
                expected_path.display(),
                Self::suggested_template_path(environment).display(),
                expected_path.display()
            ))),
            [only] if only.file_name().and_then(|name| name.to_str()) == Some(expected_name.as_str()) => {
                Ok(only.clone())
            }
            [only] => Err(ConfigError::Message(format!(
                "当前 APP_ENV={}，因此运行时配置必须是 {}。但 config/ 下唯一存在的 TOML 是 {}。请改为只保留 {}，或调整 APP_ENV。",
                environment,
                expected_name,
                only.display(),
                expected_name
            ))),
            _ => Err(ConfigError::Message(format!(
                "config/ 目录下只能存在一个运行时 TOML 文件，且文件名只能是 development.toml 或 production.toml。当前发现: {}",
                runtime_configs
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))),
        }
    }

    fn collect_runtime_configs(config_dir: &Path) -> Result<Vec<PathBuf>, ConfigError> {
        let entries = fs::read_dir(config_dir).map_err(|error| {
            ConfigError::Message(format!(
                "读取配置目录失败: dir={}, error={}",
                config_dir.display(),
                error
            ))
        })?;

        let mut runtime_configs = Vec::new();
        let mut invalid_tomls = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|error| {
                ConfigError::Message(format!(
                    "读取配置目录条目失败: dir={}, error={}",
                    config_dir.display(),
                    error
                ))
            })?;
            let path = entry.path();

            if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
                continue;
            }

            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            if RUNTIME_CONFIG_FILENAMES.contains(&file_name) {
                runtime_configs.push(path);
            } else {
                invalid_tomls.push(path);
            }
        }

        runtime_configs.sort();
        invalid_tomls.sort();

        if invalid_tomls.is_empty() {
            return Ok(runtime_configs);
        }

        Err(ConfigError::Message(format!(
            "config/ 目录下只允许保留一个运行时 TOML 文件，且名称只能是 development.toml 或 production.toml。检测到不受支持的 TOML: {}",
            invalid_tomls
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )))
    }

    fn runtime_config_filename(environment: &str) -> String {
        format!("{}.toml", environment)
    }

    fn runtime_config_path(environment: &str) -> PathBuf {
        Path::new(CONFIG_DIR).join(Self::runtime_config_filename(environment))
    }

    fn suggested_template_path(environment: &str) -> PathBuf {
        let template_name = if environment.eq_ignore_ascii_case("production") {
            "production.toml.example"
        } else {
            "development.toml.example"
        };

        Path::new(CONFIG_DIR).join(template_name)
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
            auth: AuthConfig::default(),
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
            cors: CorsConfig::default(),
            rate_limit: RateLimitConfig {
                insert_per_second: 10,
                query_per_second: 1,
            },
            room_sync: RoomSyncConfig::default(),
            electricity_fetcher: ElectricityFetcherConfig::default(),
            qq_bot: QQBotConfig::default(),
            captcha: CaptchaConfig::default(),
            verification: VerificationConfig::default(),
            notification: NotificationConfig::default(),
            email: EmailConfig::default(),
            admin: AdminConfig::default(),
            public_site: PublicSiteConfig::default(),
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
            auth: AuthConfig::default(),
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
            cors: CorsConfig::default(),
            rate_limit: RateLimitConfig {
                insert_per_second: 10,
                query_per_second: 1,
            },
            room_sync: RoomSyncConfig::default(),
            electricity_fetcher: ElectricityFetcherConfig::default(),
            qq_bot: QQBotConfig::default(),
            captcha: CaptchaConfig::default(),
            verification: VerificationConfig::default(),
            notification: NotificationConfig::default(),
            email: EmailConfig::default(),
            admin: AdminConfig::default(),
            public_site: PublicSiteConfig::default(),
            static_files: StaticFilesConfig::default(),
        };

        let err = AppConfig::validate_environment_rules(&config, "development")
            .expect_err("development 环境应拒绝远程数据库");

        assert!(err
            .to_string()
            .contains("database.host=db.example.internal"));
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
            auth: AuthConfig::default(),
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
            cors: CorsConfig::default(),
            rate_limit: RateLimitConfig {
                insert_per_second: 10,
                query_per_second: 1,
            },
            room_sync: RoomSyncConfig::default(),
            electricity_fetcher: ElectricityFetcherConfig::default(),
            qq_bot: QQBotConfig::default(),
            captcha: CaptchaConfig::default(),
            verification: VerificationConfig::default(),
            notification: NotificationConfig::default(),
            email: EmailConfig::default(),
            admin: AdminConfig::default(),
            public_site: PublicSiteConfig::default(),
            static_files: StaticFilesConfig::default(),
        };

        AppConfig::validate_environment_rules(&config, "development")
            .expect("development 环境应允许本地数据库和 Redis");
    }

    #[test]
    fn test_environment_variables_override_nested_keys_with_double_underscore() {
        let mut env = HashMap::new();
        env.insert(
            "APP__DATABASE__PASSWORD".to_string(),
            "test-password".to_string(),
        );
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
        env.insert(
            "APP__DATABASE__PASSWORD".to_string(),
            "test-password".to_string(),
        );
        env.insert("APP__DATABASE__PORT".to_string(), "15432".to_string());
        env.insert("APP__REDIS__PORT".to_string(), "16379".to_string());
        env.insert("APP__EMAIL__SMTP_PORT".to_string(), "587".to_string());
        env.insert("APP__EMAIL__SMTP_USE_TLS".to_string(), "false".to_string());
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
        assert_eq!(config.email.smtp_port, 587);
        assert!(!config.email.smtp_use_tls);
        assert_eq!(config.database.max_connections, 11);
        assert_eq!(config.jwt.expiration_hours, 48);
        assert!(!config.room_sync.enabled);
    }

    #[test]
    fn test_environment_variables_preserve_numeric_like_string_values() {
        let mut env = HashMap::new();
        env.insert(
            "APP__DATABASE__PASSWORD".to_string(),
            "test-password".to_string(),
        );
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
    fn test_public_site_config_requires_domain_and_port() {
        let empty_domain = PublicSiteConfig {
            domain: "".to_string(),
            port: "11451".to_string(),
        };
        let err = AppConfig::validate_public_site_config(&empty_domain)
            .expect_err("公开访问域名不能为空");
        assert!(err.to_string().contains("public_site.domain"));

        let empty_port = PublicSiteConfig {
            domain: "pythonrust.icu".to_string(),
            port: "".to_string(),
        };
        let err =
            AppConfig::validate_public_site_config(&empty_port).expect_err("公开访问端口不能为空");
        assert!(err.to_string().contains("public_site.port"));
    }

    #[test]
    fn test_public_site_config_rejects_url_like_domain_and_invalid_port() {
        let url_like_domain = PublicSiteConfig {
            domain: "https://pythonrust.icu".to_string(),
            port: "11451".to_string(),
        };
        let err = AppConfig::validate_public_site_config(&url_like_domain)
            .expect_err("域名字段不能包含协议");
        assert!(err.to_string().contains("不能包含协议"));

        let invalid_port = PublicSiteConfig {
            domain: "pythonrust.icu".to_string(),
            port: "65536".to_string(),
        };
        let err = AppConfig::validate_public_site_config(&invalid_port).expect_err("端口必须合法");
        assert!(err.to_string().contains("1 到 65535"));
    }

    #[test]
    fn test_qq_bot_runtime_config_requires_real_api_url_and_token() {
        let missing_api_url = QQBotConfig {
            api_url: "".to_string(),
            public_qq_number: "100000002".to_string(),
            bearer_token: "test-token".to_string(),
            bearer_token_file: None,
            timeout_seconds: 10,
        };
        let err = AppConfig::validate_qq_bot_runtime_config(&missing_api_url)
            .expect_err("QQ 机器人 API 地址不能为空");
        assert!(err.to_string().contains("qq_bot.api_url"));

        let placeholder_api_url = QQBotConfig {
            api_url: QQ_BOT_API_URL_PLACEHOLDER.to_string(),
            public_qq_number: "100000002".to_string(),
            bearer_token: "test-token".to_string(),
            bearer_token_file: None,
            timeout_seconds: 10,
        };
        let err = AppConfig::validate_qq_bot_runtime_config(&placeholder_api_url)
            .expect_err("QQ 机器人 API 地址不能保留模板占位");
        assert!(err.to_string().contains("qq_bot.api_url"));

        let missing_token = QQBotConfig {
            api_url: "http://127.0.0.1:3000/send_private_msg".to_string(),
            public_qq_number: "100000002".to_string(),
            bearer_token: "".to_string(),
            bearer_token_file: None,
            timeout_seconds: 10,
        };
        let err = AppConfig::validate_qq_bot_runtime_config(&missing_token)
            .expect_err("QQ 机器人 token 不能为空");
        assert!(err.to_string().contains("qq_bot.bearer_token"));
    }

    #[test]
    fn test_qq_bot_runtime_config_rejects_invalid_url_and_timeout() {
        let invalid_url = QQBotConfig {
            api_url: "127.0.0.1:3000/send_private_msg".to_string(),
            public_qq_number: "100000002".to_string(),
            bearer_token: "test-token".to_string(),
            bearer_token_file: None,
            timeout_seconds: 10,
        };
        let err = AppConfig::validate_qq_bot_runtime_config(&invalid_url)
            .expect_err("QQ 机器人 API 地址必须带协议");
        assert!(err.to_string().contains("http://"));

        let invalid_timeout = QQBotConfig {
            api_url: "http://127.0.0.1:3000/send_private_msg".to_string(),
            public_qq_number: "100000002".to_string(),
            bearer_token: "test-token".to_string(),
            bearer_token_file: None,
            timeout_seconds: 0,
        };
        let err = AppConfig::validate_qq_bot_runtime_config(&invalid_timeout)
            .expect_err("QQ 机器人请求超时必须大于 0");
        assert!(err.to_string().contains("qq_bot.timeout_seconds"));
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
        let email_password_path = temp_dir.join("email_password");

        fs::write(&jwt_secret_path, "secret-from-file\n").unwrap();
        fs::write(&db_password_path, "db-password-from-file\n").unwrap();
        fs::write(&qq_token_path, "qq-token-from-file\n").unwrap();
        fs::write(&email_password_path, "email-password-from-file\n").unwrap();

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
        env.insert(
            "APP__EMAIL__SMTP_PASSWORD_FILE".to_string(),
            email_password_path.to_string_lossy().to_string(),
        );

        let config = AppConfig::load_for_environment_with_source("development", Some(env))
            .expect("secret file 覆盖应成功");

        assert_eq!(config.jwt.secret, "secret-from-file");
        assert_eq!(config.database.password, "db-password-from-file");
        assert_eq!(config.qq_bot.bearer_token, "qq-token-from-file");
        assert_eq!(config.email.smtp_password, "email-password-from-file");

        let _ = fs::remove_file(jwt_secret_path);
        let _ = fs::remove_file(db_password_path);
        let _ = fs::remove_file(qq_token_path);
        let _ = fs::remove_file(email_password_path);
        let _ = fs::remove_dir(temp_dir);
    }

    #[test]
    fn test_development_requires_explicit_database_password_when_empty() {
        let mut env = HashMap::new();
        env.insert("APP__DATABASE__PASSWORD".to_string(), "".to_string());
        let err = AppConfig::load_for_environment_with_source("development", Some(env))
            .expect_err("development 环境的空数据库密码应直接失败");

        assert!(err.to_string().contains("database.password"));
    }

    #[test]
    fn test_development_rejects_password_placeholder() {
        let mut env = HashMap::new();
        env.insert(
            "APP__DATABASE__PASSWORD".to_string(),
            DEVELOPMENT_DATABASE_PASSWORD_PLACEHOLDER.to_string(),
        );
        let err = AppConfig::load_for_environment_with_source("development", Some(env))
            .expect_err("development 环境的占位密码应直接失败");

        assert!(err.to_string().contains("当前本地 PostgreSQL 的真实密码"));
    }

    #[test]
    fn test_production_requires_secret_files() {
        let config = AppConfig {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8000,
            },
            database: DatabaseConfig {
                db_type: DatabaseType::Postgres,
                host: "db.example.internal".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "".to_string(),
                password_file: None,
                database: "electricity_pro".to_string(),
                max_connections: 20,
                min_connections: 5,
                connection_timeout: 30,
            },
            jwt: JwtConfig {
                secret: "".to_string(),
                secret_file: None,
                expiration_hours: 24,
            },
            auth: AuthConfig {
                refresh_expiration_hours: 24 * 7,
                refresh_cookie_secure: true,
                refresh_cookie_same_site: "lax".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
            redis: RedisConfig {
                host: "redis.example.internal".to_string(),
                port: 6379,
                max_connections: 10,
                min_connections: 2,
                connection_timeout: 30,
            },
            cors: CorsConfig {
                allowed_origins: "https://frontend.example.com".to_string(),
            },
            rate_limit: RateLimitConfig {
                insert_per_second: 200,
                query_per_second: 200,
            },
            room_sync: RoomSyncConfig::default(),
            electricity_fetcher: ElectricityFetcherConfig::default(),
            qq_bot: QQBotConfig::default(),
            captcha: CaptchaConfig::default(),
            verification: VerificationConfig::default(),
            notification: NotificationConfig::default(),
            email: EmailConfig::default(),
            admin: AdminConfig {
                default_qq_number: "100000001".to_string(),
            },
            public_site: PublicSiteConfig::default(),
            static_files: StaticFilesConfig::default(),
        };

        let err = AppConfig::validate_sensitive_config(&config, "production")
            .expect_err("production 环境缺少 secret file 时应失败");

        assert!(err.to_string().contains("Compose secrets"));
    }

    #[test]
    fn test_production_requires_email_secret_file_when_email_is_configured() {
        let config = AppConfig {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8000,
            },
            database: DatabaseConfig {
                db_type: DatabaseType::Postgres,
                host: "db.example.internal".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "secret".to_string(),
                password_file: Some("/run/secrets/db_password".to_string()),
                database: "electricity_pro".to_string(),
                max_connections: 20,
                min_connections: 5,
                connection_timeout: 30,
            },
            jwt: JwtConfig {
                secret: "jwt-secret".to_string(),
                secret_file: Some("/run/secrets/jwt_secret".to_string()),
                expiration_hours: 24,
            },
            auth: AuthConfig {
                refresh_expiration_hours: 24 * 7,
                refresh_cookie_secure: true,
                refresh_cookie_same_site: "lax".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
            redis: RedisConfig {
                host: "redis.example.internal".to_string(),
                port: 6379,
                max_connections: 10,
                min_connections: 2,
                connection_timeout: 30,
            },
            cors: CorsConfig {
                allowed_origins: "https://frontend.example.com".to_string(),
            },
            rate_limit: RateLimitConfig {
                insert_per_second: 200,
                query_per_second: 200,
            },
            room_sync: RoomSyncConfig::default(),
            electricity_fetcher: ElectricityFetcherConfig::default(),
            qq_bot: QQBotConfig {
                bearer_token: "qq-token".to_string(),
                bearer_token_file: Some("/run/secrets/qq_token".to_string()),
                public_qq_number: "100000002".to_string(),
                ..QQBotConfig::default()
            },
            captcha: CaptchaConfig::default(),
            verification: VerificationConfig::default(),
            notification: NotificationConfig::default(),
            email: EmailConfig {
                smtp_host: "smtp.qq.com".to_string(),
                smtp_user: "cogniaegis@qq.com".to_string(),
                smtp_password: "".to_string(),
                smtp_password_file: None,
                ..EmailConfig::default()
            },
            admin: AdminConfig {
                default_qq_number: "100000001".to_string(),
            },
            public_site: PublicSiteConfig::default(),
            static_files: StaticFilesConfig::default(),
        };

        let err = AppConfig::validate_sensitive_config(&config, "production")
            .expect_err("配置 email SMTP 时 production 必须提供 smtp_password_file");

        assert!(err.to_string().contains("email.smtp_password_file"));
    }

    #[test]
    fn test_production_rejects_placeholder_admin_qq() {
        let config = AppConfig {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8000,
            },
            database: DatabaseConfig {
                db_type: DatabaseType::Postgres,
                host: "db.example.internal".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "secret".to_string(),
                password_file: Some("/run/secrets/db_password".to_string()),
                database: "electricity_pro".to_string(),
                max_connections: 20,
                min_connections: 5,
                connection_timeout: 30,
            },
            jwt: JwtConfig {
                secret: "jwt-secret".to_string(),
                secret_file: Some("/run/secrets/jwt_secret".to_string()),
                expiration_hours: 24,
            },
            auth: AuthConfig {
                refresh_expiration_hours: 24 * 7,
                refresh_cookie_secure: true,
                refresh_cookie_same_site: "lax".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
            redis: RedisConfig {
                host: "redis.example.internal".to_string(),
                port: 6379,
                max_connections: 10,
                min_connections: 2,
                connection_timeout: 30,
            },
            cors: CorsConfig {
                allowed_origins: "https://frontend.example.com".to_string(),
            },
            rate_limit: RateLimitConfig {
                insert_per_second: 200,
                query_per_second: 200,
            },
            room_sync: RoomSyncConfig::default(),
            electricity_fetcher: ElectricityFetcherConfig::default(),
            qq_bot: QQBotConfig {
                bearer_token: "qq-token".to_string(),
                bearer_token_file: Some("/run/secrets/qq_token".to_string()),
                public_qq_number: "100000002".to_string(),
                ..QQBotConfig::default()
            },
            captcha: CaptchaConfig::default(),
            verification: VerificationConfig::default(),
            notification: NotificationConfig::default(),
            email: EmailConfig::default(),
            admin: AdminConfig {
                default_qq_number: ADMIN_QQ_PLACEHOLDER.to_string(),
            },
            public_site: PublicSiteConfig::default(),
            static_files: StaticFilesConfig::default(),
        };

        let err = AppConfig::validate_security_contracts(&config, "production")
            .expect_err("production 环境应拒绝占位管理员 QQ");

        assert!(err.to_string().contains("admin.default_qq_number"));
    }

    #[test]
    fn test_production_rejects_placeholder_cors_origin() {
        let config = AppConfig {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8000,
            },
            database: DatabaseConfig {
                db_type: DatabaseType::Postgres,
                host: "db.example.internal".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "secret".to_string(),
                password_file: Some("/run/secrets/db_password".to_string()),
                database: "electricity_pro".to_string(),
                max_connections: 20,
                min_connections: 5,
                connection_timeout: 30,
            },
            jwt: JwtConfig {
                secret: "jwt-secret".to_string(),
                secret_file: Some("/run/secrets/jwt_secret".to_string()),
                expiration_hours: 24,
            },
            auth: AuthConfig {
                refresh_expiration_hours: 24 * 7,
                refresh_cookie_secure: true,
                refresh_cookie_same_site: "lax".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
            redis: RedisConfig {
                host: "redis.example.internal".to_string(),
                port: 6379,
                max_connections: 10,
                min_connections: 2,
                connection_timeout: 30,
            },
            cors: CorsConfig {
                allowed_origins: CORS_ALLOW_ORIGINS_PLACEHOLDER.to_string(),
            },
            rate_limit: RateLimitConfig {
                insert_per_second: 200,
                query_per_second: 200,
            },
            room_sync: RoomSyncConfig::default(),
            electricity_fetcher: ElectricityFetcherConfig::default(),
            qq_bot: QQBotConfig {
                bearer_token: "qq-token".to_string(),
                bearer_token_file: Some("/run/secrets/qq_token".to_string()),
                public_qq_number: "100000002".to_string(),
                ..QQBotConfig::default()
            },
            captcha: CaptchaConfig::default(),
            verification: VerificationConfig::default(),
            notification: NotificationConfig::default(),
            email: EmailConfig::default(),
            admin: AdminConfig {
                default_qq_number: "100000001".to_string(),
            },
            public_site: PublicSiteConfig::default(),
            static_files: StaticFilesConfig::default(),
        };

        let err = AppConfig::validate_security_contracts(&config, "production")
            .expect_err("production 环境应拒绝占位 CORS Origin");

        assert!(err.to_string().contains("cors.allowed_origins"));
    }

    #[test]
    fn test_missing_runtime_config_has_friendly_message() {
        let temp_dir = std::env::temp_dir().join(format!(
            "electricity-monitor-missing-config-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(temp_dir.join("config")).unwrap();

        let err =
            AppConfig::load_for_environment_in_dir("development", Some(HashMap::new()), &temp_dir)
                .expect_err("缺少 development.toml 时应返回友好错误");

        assert!(err.to_string().contains("development.toml"));
        assert!(err.to_string().contains("development.toml.example"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_runtime_config_must_match_environment() {
        let temp_dir = std::env::temp_dir().join(format!(
            "electricity-monitor-config-env-mismatch-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp_dir);
        let config_dir = temp_dir.join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("production.toml"), "").unwrap();

        let err =
            AppConfig::load_for_environment_in_dir("development", Some(HashMap::new()), &temp_dir)
                .expect_err("development 环境不应读取 production.toml");

        assert!(err.to_string().contains("APP_ENV=development"));
        assert!(err.to_string().contains("production.toml"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_runtime_config_rejects_multiple_runtime_tomls() {
        let temp_dir = std::env::temp_dir().join(format!(
            "electricity-monitor-config-multiple-runtime-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp_dir);
        let config_dir = temp_dir.join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("development.toml"), "").unwrap();
        fs::write(config_dir.join("production.toml"), "").unwrap();

        let err =
            AppConfig::load_for_environment_in_dir("development", Some(HashMap::new()), &temp_dir)
                .expect_err("config/ 下同时存在 development.toml 与 production.toml 时应失败");

        assert!(err.to_string().contains("只能存在一个运行时 TOML"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_runtime_config_rejects_unsupported_runtime_toml_name() {
        let temp_dir = std::env::temp_dir().join(format!(
            "electricity-monitor-config-unsupported-runtime-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp_dir);
        let config_dir = temp_dir.join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("legacy-runtime.toml"), "").unwrap();

        let err =
            AppConfig::load_for_environment_in_dir("development", Some(HashMap::new()), &temp_dir)
                .expect_err("不受支持的运行时 TOML 文件名应被拒绝");

        assert!(err.to_string().contains("不受支持的 TOML"));
        assert!(err.to_string().contains("development.toml"));
        assert!(err.to_string().contains("production.toml"));

        let _ = fs::remove_dir_all(temp_dir);
    }
}
