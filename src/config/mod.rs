//! 配置管理模块
//!
//! 负责加载和管理应用程序配置，支持多环境配置 (development/production)

pub mod app;
pub mod database;

pub use app::AppConfig;
pub use database::DatabaseConfig;
