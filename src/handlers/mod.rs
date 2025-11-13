//! HTTP 处理器 (Handlers)
//!
//! 处理HTTP请求并返回响应

pub mod auth;
pub mod binding;
pub mod electricity_fetcher;
pub mod health;
pub mod room;
pub mod room_sync;

pub use auth::*;
pub use binding::*;
pub use electricity_fetcher::*;
pub use health::{health_check, health_check_db};
pub use room::*;
pub use room_sync::*;
