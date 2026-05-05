//! 配置管理模块
//!
//! 负责加载和管理应用程序配置，支持多环境配置 (development/production)

pub mod admin;
pub mod app;
pub mod auth;
pub mod captcha;
pub mod cors;
pub mod database;
pub mod electricity_fetcher;
pub mod notification;
pub mod qq_bot;
pub mod rate_limit;
pub mod redis;
pub mod room_sync;
pub mod static_files;
pub mod verification;

pub use admin::AdminConfig;
pub use app::{AppConfig, JwtConfig, LoggingConfig, ServerConfig};
pub use auth::AuthConfig;
pub use captcha::CaptchaConfig;
pub use cors::CorsConfig;
pub use database::DatabaseConfig;
pub use electricity_fetcher::ElectricityFetcherConfig;
pub use notification::NotificationConfig;
pub use qq_bot::QQBotConfig;
pub use rate_limit::RateLimitConfig;
pub use redis::RedisConfig;
pub use room_sync::{CrawlerConfig, RoomSyncConfig};
pub use static_files::StaticFilesConfig;
pub use verification::VerificationConfig;
