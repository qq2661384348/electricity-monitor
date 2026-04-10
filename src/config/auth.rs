//! 认证相关运行时配置

use serde::Deserialize;

const DEFAULT_REFRESH_EXPIRATION_HOURS: u64 = 24 * 7;
const DEFAULT_REFRESH_COOKIE_SAME_SITE: &str = "lax";

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    #[serde(default = "default_refresh_expiration_hours")]
    pub refresh_expiration_hours: u64,

    #[serde(default)]
    pub refresh_cookie_secure: bool,

    #[serde(default = "default_refresh_cookie_same_site")]
    pub refresh_cookie_same_site: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            refresh_expiration_hours: default_refresh_expiration_hours(),
            refresh_cookie_secure: false,
            refresh_cookie_same_site: default_refresh_cookie_same_site(),
        }
    }
}

fn default_refresh_expiration_hours() -> u64 {
    DEFAULT_REFRESH_EXPIRATION_HOURS
}

fn default_refresh_cookie_same_site() -> String {
    DEFAULT_REFRESH_COOKIE_SAME_SITE.to_string()
}
