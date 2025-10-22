//! HTTP 处理器 (Handlers)
//!
//! 处理HTTP请求并返回响应

pub mod health;

pub use health::{health_check, health_check_db};
