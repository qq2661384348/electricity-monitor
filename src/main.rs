//! Electricity Monitor Backend Server
//!
//! 高性能电力监控系统后端服务器入口

// 设置 mimalloc 作为全局内存分配器
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use axum::Router;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use electricity_monitor_backend::{
    config::AppConfig,
    infrastructure::database::create_pool,
    middleware::logger::create_trace_layer,
    routes::create_routes,
    state::AppState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    init_tracing();

    tracing::info!("Starting Electricity Monitor Backend...");

    // 加载配置
    AppConfig::init()?;
    let config = AppConfig::global();

    tracing::info!(
        "Configuration loaded: environment={}, database={:?}",
        std::env::var("APP_ENV").unwrap_or("development".to_string()),
        config.database.db_type
    );

    // 创建数据库连接池
    let db_pool = create_pool(&config.database).await?;
    tracing::info!("Database pool created successfully");

    // 创建应用状态
    let state = AppState::new(db_pool);

    // 构建应用路由
    let app = create_app(state);

    // 服务器地址
    let addr: SocketAddr = config.server_addr().parse()?;
    tracing::info!("Server listening on {}", addr);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 创建Axum应用
fn create_app(state: AppState) -> Router {
    // CORS配置
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 构建路由
    Router::new()
        .merge(create_routes())
        .layer(
            ServiceBuilder::new()
                .layer(create_trace_layer())  // 日志跟踪
                .layer(cors)                   // CORS
        )
        .with_state(state)
}

/// 初始化日志追踪
fn init_tracing() {
    // 从环境变量获取日志级别，默认为info
    let log_level = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info,electricity_monitor_backend=debug".to_string());

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_level.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
