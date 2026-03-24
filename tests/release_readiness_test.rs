use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::{Arc, Once};
use tower::ServiceExt;

use electricity_monitor_backend::{
    bootstrap::router::create_app,
    config::AppConfig,
    domain::services::RateLimiter,
    infrastructure::{database::pool::create_pool, redis::pool::create_redis_pool, CacheManager, CacheManagerConfig},
    state::AppState,
};

static INIT: Once = Once::new();

fn ensure_config_init() {
    INIT.call_once(|| {
        AppConfig::init().expect("配置初始化失败");
    });
}

async fn create_test_app() -> axum::Router {
    ensure_config_init();
    let config = AppConfig::global();

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
    create_app(state)
}

#[tokio::test]
async fn test_health_endpoints_pass() {
    let app = create_test_app().await;

    let health = app
        .clone()
        .oneshot(Request::builder().uri("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);

    let health_db = app
        .oneshot(Request::builder().uri("/api/health/db").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(health_db.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_static_index_accessible() {
    let app = create_test_app().await;

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
