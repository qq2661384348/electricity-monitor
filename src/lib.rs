//! Electricity Monitor Backend
//!
//! 高性能电力监控系统后端API

pub mod config;
pub mod domain;
pub mod errors;
pub mod handlers;
pub mod infrastructure;
pub mod middleware;
pub mod routes;
pub mod state;
pub mod utils;

pub use config::AppConfig;
pub use errors::{AppError, Result};
pub use state::AppState;
