//! Electricity Monitor Backend Server
//!
//! 高性能电力监控系统后端服务器入口

// 设置 mimalloc 作为全局内存分配器
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use axum::Router;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::sync::Arc;

use electricity_monitor_backend::{
    config::AppConfig,
    domain::services::{
        spawn_recovery_monitor_persistent, ElectricityFetcherService, ElectricityService,
        NotificationGate, NotificationService, RateLimiter, RoomSyncCache,
    },
    infrastructure::{database::create_pool, redis::create_redis_pool, repositories::RoomRepository},
    middleware::logger::create_trace_layer,
    routes::{create_routes, create_static_service},
    state::AppState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 先加载配置（日志初始化需要用到配置）
    AppConfig::init()?;
    let config = AppConfig::global();

    // 根据配置初始化日志
    init_tracing(&config.logging);

    tracing::info!("Starting Electricity Monitor Backend...");
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

    // 触发启动时房间同步（如果配置启用）
    if config.room_sync.enabled {
        use electricity_monitor_backend::domain::services::RoomSyncService;
        
        tracing::info!("检查是否需要启动时房间同步...");
        
        // 检查数据库中是否有房间数据
        let room_repo = RoomRepository::new(db_pool.clone());
        let room_count = room_repo.count_all().await?;
        
        if room_count == 0 {
            tracing::info!("数据库无房间数据，触发启动时自动同步");
            
            // 创建RoomClient和RoomFetcher
            use electricity_monitor_backend::domain::services::room_sync::crawler::{RoomClient, RoomFetcher};
            let client = Arc::new(RoomClient::new(&config.room_sync.crawler)?);
            let fetcher = Arc::new(RoomFetcher::new(client));
            
            // 创建RoomSyncCache（初始化时自动加载数据）
            let room_sync_cache = match RoomSyncCache::new(db_pool.clone()).await {
                Ok(cache) => Arc::new(cache),
                Err(e) => {
                    tracing::error!(error = %e, "创建RoomSyncCache失败，启动时同步将被跳过");
                    return Err(anyhow::anyhow!("创建RoomSyncCache失败: {}", e));
                }
            };
            
            // 创建同步服务
            let sync_service = RoomSyncService::new(
                Arc::new(db_pool.clone()),
                Arc::new(room_repo.clone()),
                fetcher,
                room_sync_cache,
                config.room_sync.default_threshold,
            );
            
            match sync_service.sync().await {
                Ok(stats) => {
                    tracing::info!(
                        total = stats.total,
                        new = stats.new,
                        updated = stats.updated,
                        failed = stats.failed,
                        "启动时房间同步完成"
                    );
                }
                Err(e) => {
                    tracing::error!(error = %e, "启动时房间同步失败，将继续启动但可能无数据");
                }
            }
        } else {
            tracing::info!(room_count = room_count, "数据库已有房间数据，跳过启动时同步");
        }
    }

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
            config.electricity_fetcher.max_retries,
            config.electricity_fetcher.retry_delay_seconds,
            config.electricity_fetcher.retry_backoff_multiplier,
        )
        .await?;
        
        scheduler.start().await?;
        tracing::info!(
            "Electricity Fetcher Service started (fetch: {}min, history: {}h, retries: {}, delay: {}s, backoff: {}x)",
            config.electricity_fetcher.fetch_interval_minutes,
            config.electricity_fetcher.history_interval_hours,
            config.electricity_fetcher.max_retries,
            config.electricity_fetcher.retry_delay_seconds,
            config.electricity_fetcher.retry_backoff_multiplier
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
    
    // 初始化路径树（从数据库加载所有房间）
    {
        use electricity_monitor_backend::domain::services::RoomPathTree;
        
        tracing::info!("正在初始化房间路径树...");
        let room_repo = RoomRepository::new(db_pool.clone());
        
        // 查询所有活跃房间
        match room_repo.find_all_active().await {
            Ok(rooms) => {
                // 转换为 RoomData 格式
                let room_data: Vec<electricity_monitor_backend::domain::services::room_sync::crawler::models::RoomData> = rooms.iter().map(|r| {
                    electricity_monitor_backend::domain::services::room_sync::crawler::models::RoomData {
                        roomid: r.roomid,
                        roompaths: vec![r.primary_roompath.clone()],
                        primary_roompath: r.primary_roompath.clone(),
                        path_count: if r.has_additional_paths { 2 } else { 1 },
                    }
                }).collect();
                
                // 构建路径树
                let tree = RoomPathTree::build_from_rooms(&room_data);
                state.update_path_tree(tree).await;
                tracing::info!("房间路径树初始化完成，包含 {} 个房间", rooms.len());
            }
            Err(e) => {
                tracing::warn!("初始化路径树失败: {}，将使用空树", e);
            }
        }
    }

    // 启动后台服务
    spawn_background_services(state.clone());

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
fn spawn_background_services(state: AppState) {
    use electricity_monitor_backend::infrastructure::repositories::{UserRepository, UserRoomBindingRepository};
    
    let db_pool = state.db_pool.clone();
    let redis_pool = state.redis_pool.clone();
    let rate_limiter = state.rate_limiter.clone();
    let flagged_rooms_cache = state.flagged_rooms_cache.clone();

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
            
            // 创建通知门控器（防抖观察期：1小时）
            let notification_gate = Arc::new(NotificationGate::new(
                Some(std::time::Duration::from_secs(config.notification.debounce_period_secs))
            ));
            tracing::info!(
                "NotificationGate created (debounce_period: {}s)",
                config.notification.debounce_period_secs
            );
            
            // 异步初始化：从数据库加载历史状态并启动监控任务
            let gate_for_init = notification_gate.clone();
            let binding_repo_for_init = binding_repository.clone();
            let room_repo_for_init = room_repository.clone();
            let recovery_interval = config.notification.recovery_monitor_interval_secs;
            
            tokio::spawn(async move {
                // 从数据库加载历史通知状态（防止重启后重复通知）
                if let Err(e) = gate_for_init.load_from_database(&binding_repo_for_init, &room_repo_for_init).await {
                    tracing::error!("Failed to load notification history from database: {}", e);
                    // 继续执行，不中断服务启动
                } else {
                    tracing::info!("Notification history loaded from database");
                }
                
                // 启动房间恢复监控任务（带持久化）
                let _recovery_monitor_handle = spawn_recovery_monitor_persistent(
                    room_repo_for_init,
                    binding_repo_for_init,
                    gate_for_init,
                    recovery_interval,
                );
                tracing::info!(
                    "Recovery monitor started with persistence (interval: {}s)",
                    recovery_interval
                );
            });
            
            // 创建通知服务
            let notification_service = NotificationService::new(
                room_repository.clone(),
                user_repository,
                binding_repository,
                qq_client,
                rate_limiter.clone(),
                notification_gate,
                &config.notification,
            );
            notification_service.spawn_worker();
            tracing::info!(
                "Notification service started (interval: {}s, concurrent: {})",
                config.notification.query_interval_secs,
                config.notification.concurrent_send_limit
            );
        }
        Err(e) => {
            tracing::error!("QQ客户端初始化失败，通知服务未启动: {}", e);
        }
    }

    // 3. 启动Flagged Rooms缓存刷新任务（每10秒刷新一次）
    // 这解决了N+1查询问题，并降低了数据库压力
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        tracing::info!("Flagged rooms cache refresher started (interval: 10s)");
        
        loop {
            interval.tick().await;
            
            // 查询数据库
            let start = std::time::Instant::now();
            match room_repository.find_rooms_with_send_flag_true().await {
                Ok(rooms) => {
                    let count = rooms.len();
                    
                    // 更新缓存
                    let mut cache = flagged_rooms_cache.write().await;
                    *cache = rooms;
                    let duration = start.elapsed();
                    // 仅在数据变化或长时间间隔时打印DEBUG日志，避免刷屏
                    // 这里每次刷新都打印DEBUG，生产环境可能需要调整日志级别
                    tracing::debug!(
                        count = count,
                        duration_ms = duration.as_millis(),
                        "Updated flagged rooms cache"
                    );
                },
                Err(e) => {
                    tracing::error!("Failed to fetch flagged rooms from DB: {}", e);
                }
            }
        }
    });
}

/// 创建Axum应用
fn create_app(state: AppState) -> Router {
    let config = AppConfig::global();
    
    // CORS配置
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 构建基础路由（API）
    let mut app = Router::new()
        .merge(create_routes())
        .with_state(state);
    
    // 如果启用静态文件服务，添加 fallback
    if config.static_files.enabled {
        if config.static_files.directory_exists() {
            let static_service = create_static_service(&config.static_files);
            app = app.fallback_service(static_service);
            tracing::info!(
                "Static file service enabled: directory={}, index={}",
                config.static_files.directory,
                config.static_files.index_file
            );
        } else {
            tracing::warn!(
                "Static file service enabled but directory '{}' does not exist, skipping",
                config.static_files.directory
            );
        }
    }
    
    // 添加全局中间件
    app.layer(
        ServiceBuilder::new()
            .layer(CompressionLayer::new())                    // Gzip/Brotli 压缩
            .layer(create_trace_layer(&config.logging.level))  // 日志跟踪
            .layer(cors)                                       // CORS
    )
}

/// 初始化日志追踪
/// 
/// # 参数
/// - `config`: 日志配置
/// 
/// # 优先级
/// 1. 环境变量 `RUST_LOG`（最高优先级）
/// 2. 配置文件 `logging.level`
fn init_tracing(config: &electricity_monitor_backend::config::LoggingConfig) {
    // 优先使用环境变量，否则使用配置文件
    let log_level = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| {
            // 使用配置文件中的日志级别
            // 设置全局默认级别，同时为项目和常见的第三方库设置级别
            // tower_http 级别跟随全局配置，控制 HTTP 访问日志
            format!(
                "{level},electricity_monitor_backend={level},tower_http={level},tokio_postgres=warn,hyper=warn",
                level = config.level
            )
        });

    // 根据配置选择日志格式
    let registry = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_level.into()),
        );

    match config.format.as_str() {
        "json" => {
            registry
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        }
        _ => {
            // 默认 pretty 格式
            registry
                .with(tracing_subscriber::fmt::layer())
                .init();
        }
    }
}
