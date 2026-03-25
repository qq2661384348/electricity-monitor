use std::sync::{Arc, Once};

use axum::Router;
use electricity_monitor_backend::{
    bootstrap::router::create_app,
    config::AppConfig,
    domain::services::RateLimiter,
    infrastructure::{
        database::pool::create_pool, redis::pool::create_redis_pool, CacheManager,
        CacheManagerConfig,
    },
    state::AppState,
};

static INIT: Once = Once::new();

pub struct TestApp {
    pub app: Router,
    pub state: AppState,
}

fn ensure_config_init() {
    INIT.call_once(|| {
        AppConfig::init().expect("配置初始化失败");
    });
}

pub fn test_config() -> &'static AppConfig {
    ensure_config_init();
    AppConfig::global()
}

pub async fn create_test_app() -> TestApp {
    let config = test_config();

    let db_pool = create_pool(&config.database)
        .await
        .expect("数据库连接池创建失败");
    let redis_pool = create_redis_pool(&config.redis)
        .await
        .expect("Redis连接池创建失败");
    let rate_limiter = Arc::new(RateLimiter::new(
        redis_pool.clone(),
        config.rate_limit.clone(),
    ));
    let cache_manager = Arc::new(CacheManager::new(
        CacheManagerConfig::default(),
        db_pool.clone(),
        Some(redis_pool.clone()),
    ));

    let state = AppState::new(db_pool, redis_pool, rate_limiter, None, cache_manager);
    let app = create_app(state.clone());

    TestApp { app, state }
}
