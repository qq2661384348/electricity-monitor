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

use std::sync::Arc;

use electricity_monitor_backend::{
    config::AppConfig,
    domain::services::{ElectricityFetcherService, ElectricityService, NotificationService, RateLimiter},
    infrastructure::{database::create_pool, redis::create_redis_pool, repositories::RoomRepository},
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

    // 创建Redis连接池
    let redis_pool = create_redis_pool(&config.redis).await?;
    tracing::info!("Redis pool created successfully");

    // 创建限流器
    let rate_limiter = Arc::new(RateLimiter::new(
        redis_pool.clone(),
        config.rate_limit.clone(),
    ));
    tracing::info!("Rate limiter initialized");

    // 初始化ElectricityFetcherService（如果启用）
    let electricity_fetcher_service = if config.electricity_fetcher.enabled {
        tracing::info!("Initializing Electricity Fetcher Service...");
        
        let service = ElectricityFetcherService::new(
            config.electricity_fetcher.api_url.clone(),
            db_pool.clone(),
            redis_pool.clone(),
        )
        .await?;
        
        let service_arc = Arc::new(service);
        
        // 启动定时任务
        let scheduler = ElectricityFetcherService::start_scheduler(
            service_arc.clone(),
            config.electricity_fetcher.fetch_interval_minutes,
            config.electricity_fetcher.history_interval_hours,
        )
        .await?;
        
        scheduler.start().await?;
        tracing::info!(
            "Electricity Fetcher Service started (fetch: {}min, history: {}h)",
            config.electricity_fetcher.fetch_interval_minutes,
            config.electricity_fetcher.history_interval_hours
        );
        
        Some(service_arc)
    } else {
        tracing::info!("Electricity Fetcher Service disabled");
        None
    };

    // 创建应用状态
    let state = AppState::new(
        db_pool.clone(),
        redis_pool.clone(),
        rate_limiter.clone(),
        electricity_fetcher_service,
    );

    // 启动后台服务
    spawn_background_services(
        db_pool.clone(),
        redis_pool.clone(),
        rate_limiter.clone(),
    );

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

/// 启动后台服务
fn spawn_background_services(
    db_pool: electricity_monitor_backend::infrastructure::DbPool,
    redis_pool: electricity_monitor_backend::infrastructure::RedisPool,
    rate_limiter: Arc<RateLimiter>,
) {
    use electricity_monitor_backend::infrastructure::repositories::{UserRepository, UserRoomBindingRepository};
    
    // 创建Repositories
    let room_repository = RoomRepository::new(db_pool.clone());
    let user_repository = UserRepository::new(db_pool.clone());
    let binding_repository = UserRoomBindingRepository::new(db_pool.clone());

    // 1. 启动电费插入服务
    let electricity_service = ElectricityService::new(
        room_repository.clone(),
        redis_pool.clone(),
        rate_limiter.clone(),
    );
    electricity_service.spawn_worker();
    tracing::info!("Electricity insertion service started");

    // 2. 创建QQ客户端并启动通知服务（每60秒查询一次）
    let config = electricity_monitor_backend::config::AppConfig::global();
    match electricity_monitor_backend::infrastructure::QQClient::new(config.qq_bot.clone()) {
        Ok(qq_client) => {
            let qq_client = Arc::new(qq_client);
            let notification_service = NotificationService::new(
                room_repository.clone(),
                user_repository,
                binding_repository,
                qq_client,
                rate_limiter.clone(),
                Some(60),
                Some(10),
            );
            notification_service.spawn_worker();
            tracing::info!("Notification service started (interval: 60s, concurrent: 10)");
        }
        Err(e) => {
            tracing::error!("QQ客户端初始化失败，通知服务未启动: {}", e);
        }
    }
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
