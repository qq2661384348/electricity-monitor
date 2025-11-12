//! 配置管理模块
//!
//! 负责加载和管理应用程序配置，支持多环境配置 (development/production)

pub mod app;
pub mod database;
pub mod electricity_fetcher;
pub mod rate_limit;
pub mod redis;
pub mod room_sync;

pub use app::AppConfig;
pub use database::DatabaseConfig;
pub use electricity_fetcher::ElectricityFetcherConfig;
pub use rate_limit::RateLimitConfig;
pub use redis::RedisConfig;
pub use room_sync::{CrawlerConfig, RoomSyncConfig};
