use axum::Router;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
};

use crate::{
    config::AppConfig,
    routes::{create_routes, create_static_service},
    state::AppState,
};

pub fn create_app(state: AppState) -> Router {
    let config = AppConfig::global();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut app = Router::new().merge(create_routes()).with_state(state);

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
            .layer(crate::middleware::logger::create_trace_layer(
                &config.logging.level,
            ))
            .layer(cors),
    )
}
