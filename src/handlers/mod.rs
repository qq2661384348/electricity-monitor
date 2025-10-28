//! HTTP 处理器 (Handlers)
//!
//! 处理HTTP请求并返回响应

pub mod health;
pub mod room;

pub use health::{health_check, health_check_db};
pub use room::*;
