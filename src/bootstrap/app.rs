use std::{net::SocketAddr, sync::Arc};

use crate::{
    bootstrap::{config, observability, router, runtime, shutdown},
    domain::services::RateLimiter,
    infrastructure::{
        database::create_pool, redis::create_redis_pool, CacheManager, CacheManagerConfig,
    },
    state::AppState,
};

pub async fn run() -> anyhow::Result<()> {
    let config = config::init()?;
    observability::init(&config.logging);

    tracing::info!("Starting Electricity Monitor Backend...");
    tracing::info!(
        "Configuration loaded: environment={}, database={:?}",
        config.environment(),
        config.database.db_type
    );

    let db_pool = create_pool(&config.app_config().database).await?;
    tracing::info!("Database pool created successfully");

    let redis_pool = create_redis_pool(&config.app_config().redis).await?;
    tracing::info!("Redis pool created successfully");

    let rate_limiter = Arc::new(RateLimiter::new(
        redis_pool.clone(),
        config.rate_limit.clone(),
    ));
    tracing::info!("Rate limiter initialized");

    let cache_manager = Arc::new(CacheManager::new(
        CacheManagerConfig::default(),
        db_pool.clone(),
        Some(redis_pool.clone()),
    ));
    tracing::info!("Cache manager initialized");

    runtime::run_startup_room_sync(&config, &db_pool).await?;

    let electricity_fetcher_service =
        runtime::initialize_electricity_fetcher_service(&config, &db_pool, &redis_pool).await?;
    let email_sender = runtime::initialize_email_sender(&config)?;

    let state = AppState::new(
        db_pool.clone(),
        redis_pool.clone(),
        rate_limiter,
        electricity_fetcher_service,
        cache_manager.clone(),
    )
    .with_email_sender(email_sender);

    runtime::initialize_path_tree(&state, &db_pool).await;
    runtime::spawn_background_services(state.clone());

    let app = router::create_app(state);
    let addr: SocketAddr = config.server_addr().parse()?;
    tracing::info!("Server listening on {}", addr);

    shutdown::serve(addr, app).await
}
