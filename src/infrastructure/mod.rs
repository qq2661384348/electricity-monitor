//! 基础设施层 (Infrastructure Layer)
//!
//! 提供数据库、外部服务等基础设施支持

pub mod database;
pub mod electricity;
pub mod redis;
pub mod repositories;

pub use database::DbPool;
pub use redis::RedisPool;
