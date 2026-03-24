use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct HttpClientConfig {
    pub timeout: Option<Duration>,
    pub connect_timeout: Option<Duration>,
    pub danger_accept_invalid_certs: bool,
    pub pool_max_idle_per_host: Option<usize>,
    pub pool_idle_timeout: Option<Duration>,
    pub tcp_keepalive: Option<Duration>,
}

pub fn build_reqwest_client(config: &HttpClientConfig) -> Result<Client, reqwest::Error> {
    let mut builder = Client::builder();

    if let Some(timeout) = config.timeout {
        builder = builder.timeout(timeout);
    }
    if let Some(connect_timeout) = config.connect_timeout {
        builder = builder.connect_timeout(connect_timeout);
    }
    if let Some(pool_max_idle) = config.pool_max_idle_per_host {
        builder = builder.pool_max_idle_per_host(pool_max_idle);
    }
    if let Some(pool_idle_timeout) = config.pool_idle_timeout {
        builder = builder.pool_idle_timeout(pool_idle_timeout);
    }
    if let Some(tcp_keepalive) = config.tcp_keepalive {
        builder = builder.tcp_keepalive(tcp_keepalive);
    }

    builder
        .danger_accept_invalid_certs(config.danger_accept_invalid_certs)
        .build()
}

pub fn http_status_error_message(external_dependency: &str, status: reqwest::StatusCode) -> String {
    format!(
        "external_dependency={}, status={}",
        external_dependency, status
    )
}
