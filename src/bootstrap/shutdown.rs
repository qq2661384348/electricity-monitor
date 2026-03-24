use std::net::SocketAddr;

use axum::Router;

pub async fn serve(listener_addr: SocketAddr, app: Router) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(listener_addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(wait_for_shutdown_signal())
        .await?;
    Ok(())
}

async fn wait_for_shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install terminate handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    tracing::info!("Shutdown signal received");
}
