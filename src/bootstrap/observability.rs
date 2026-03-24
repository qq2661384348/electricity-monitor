use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::LoggingConfig;

pub fn init(config: &LoggingConfig) {
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        format!(
            "{level},electricity_monitor_backend={level},tower_http={level},tokio_postgres=warn,hyper=warn",
            level = config.level
        )
    });

    let registry = tracing_subscriber::registry().with(
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.into()),
    );

    match config.format.as_str() {
        "json" => {
            registry
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        }
        _ => {
            registry.with(tracing_subscriber::fmt::layer()).init();
        }
    }
}
