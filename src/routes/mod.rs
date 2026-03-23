//! 路由定义
//!
//! 定义所有API路由和静态文件服务

use crate::state::AppState;
use axum::Router;

pub mod api;
pub mod auth;
pub mod binding;
pub mod captcha;
pub mod electricity_fetcher;
pub mod room;
pub mod room_sync;
pub mod static_files;

pub use static_files::create_static_service;

/// 创建应用路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .nest("/api", api::routes())
        .nest("/api", auth::routes())
        .nest("/api", binding::routes())
        .nest("/api", captcha::routes())
        .nest("/api", room::routes())
        .nest("/api", room_sync::routes())
        .nest("/api", electricity_fetcher::routes())
}
