//! CORS 运行时配置

use serde::Deserialize;

const DEFAULT_ALLOWED_ORIGINS: &str = "http://localhost:5173";

#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    #[serde(default = "default_allowed_origins")]
    pub allowed_origins: String,
}

impl CorsConfig {
    pub fn origin_list(&self) -> Vec<String> {
        self.allowed_origins
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: default_allowed_origins(),
        }
    }
}

fn default_allowed_origins() -> String {
    DEFAULT_ALLOWED_ORIGINS.to_string()
}
