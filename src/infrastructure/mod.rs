//! 基础设施层 (Infrastructure Layer)
//!
//! 提供数据库、外部服务等基础设施支持

pub mod cache;
pub mod database;
pub mod electricity;
pub mod notification;
pub mod redis;
pub mod repositories;

pub use cache::{SimpleCacheConfig, SimpleCacheManager};
pub use database::DbPool;
pub use notification::QQClient;
pub use redis::RedisPool;
