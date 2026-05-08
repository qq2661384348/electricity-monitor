use axum::http::{header, HeaderValue, Method};
use axum::{middleware, Router};
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, cors::CorsLayer};

use crate::{
    config::AppConfig,
    routes::{create_routes, create_static_service},
    state::AppState,
};

pub fn create_app(state: AppState) -> Router {
    let config = AppConfig::global();
    let allowed_origins = config
        .cors
        .origin_list()
        .into_iter()
        .map(|origin| {
            HeaderValue::from_str(&origin)
                .expect("cors.allowed_origins 应在配置校验阶段保证为合法 HeaderValue")
        })
        .collect::<Vec<_>>();

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_credentials(true);

    let mut app = Router::new()
        .merge(create_routes(state.clone()))
        .with_state(state);

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

    app.layer(
        ServiceBuilder::new()
            .layer(CompressionLayer::new())
            .layer(middleware::from_fn(
                crate::middleware::security_headers::apply,
            ))
            .layer(crate::middleware::logger::create_trace_layer(
                &config.logging.level,
            ))
            .layer(cors),
    )
}
