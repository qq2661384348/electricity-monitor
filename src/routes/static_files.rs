//! 静态文件服务
//!
//! 提供前端静态资源的分发服务，支持：
//! - SPA 单页应用路由 fallback
//! - 智能缓存头（带 hash 的资源长缓存，HTML 无缓存）
//! - Gzip/Brotli 压缩

use axum::{
    http::{header, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_status::SetStatus;

use crate::config::StaticFilesConfig;

/// 创建静态文件服务
///
/// # 功能
/// - 服务 `directory` 目录下的所有静态文件
/// - 未找到文件时 fallback 到 `index.html`（支持 SPA 路由）
///
/// # 参数
/// - `config`: 静态文件配置
///
/// # 返回
/// - `ServeDir` 服务，可用于 `Router::fallback_service`
pub fn create_static_service(config: &StaticFilesConfig) -> ServeDir<SetStatus<ServeFile>> {
    let index_path = config.index_path();

    ServeDir::new(&config.directory).not_found_service(ServeFile::new(index_path))
}

/// 缓存控制中间件响应头值
pub struct CacheHeaders;

impl CacheHeaders {
    /// 带 hash 的资源（如 xxx-abc123.js）使用长期缓存
    /// immutable 表示内容永远不变，浏览器无需重新验证
    pub fn immutable(max_age_seconds: u64) -> HeaderValue {
        HeaderValue::from_str(&format!("public, max-age={}, immutable", max_age_seconds))
            .unwrap_or_else(|_| HeaderValue::from_static("public, max-age=31536000, immutable"))
    }

    /// HTML 文件使用 no-cache，每次都验证
    pub fn no_cache() -> HeaderValue {
        HeaderValue::from_static("no-cache, no-store, must-revalidate")
    }

    /// 短期缓存（如 favicon 等）
    pub fn short_cache(seconds: u64) -> HeaderValue {
        HeaderValue::from_str(&format!("public, max-age={}", seconds))
            .unwrap_or_else(|_| HeaderValue::from_static("public, max-age=3600"))
    }
}

/// 根据文件路径判断缓存策略
///
/// # 规则
/// - `.html` 文件：no-cache
/// - `assets/` 目录下带 hash 的文件：1年强缓存
/// - 其他文件：1小时缓存
pub fn get_cache_control(uri: &Uri, config: &StaticFilesConfig) -> HeaderValue {
    let path = uri.path();

    // HTML 文件不缓存
    if path.ends_with(".html") || path == "/" {
        return CacheHeaders::no_cache();
    }

    // assets 目录下的文件使用长期缓存（假设都带 hash）
    if path.contains("/assets/") {
        return CacheHeaders::immutable(config.cache_max_age_seconds);
    }

    // 其他文件使用短期缓存
    CacheHeaders::short_cache(3600)
}

/// 404 响应处理器
pub async fn not_found_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}

/// 健康检查（静态文件服务）
pub async fn static_health_check() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .body("Static file service is running".to_string())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_headers_immutable() {
        let header = CacheHeaders::immutable(31536000);
        assert!(header.to_str().unwrap().contains("immutable"));
        assert!(header.to_str().unwrap().contains("31536000"));
    }

    #[test]
    fn test_cache_headers_no_cache() {
        let header = CacheHeaders::no_cache();
        assert!(header.to_str().unwrap().contains("no-cache"));
    }

    #[test]
    fn test_get_cache_control_html() {
        let config = StaticFilesConfig::default();
        let uri: Uri = "/index.html".parse().unwrap();
        let header = get_cache_control(&uri, &config);
        assert!(header.to_str().unwrap().contains("no-cache"));
    }

    #[test]
    fn test_get_cache_control_assets() {
        let config = StaticFilesConfig::default();
        let uri: Uri = "/assets/index-abc123.js".parse().unwrap();
        let header = get_cache_control(&uri, &config);
        assert!(header.to_str().unwrap().contains("immutable"));
    }

    #[test]
    fn test_get_cache_control_root() {
        let config = StaticFilesConfig::default();
        let uri: Uri = "/".parse().unwrap();
        let header = get_cache_control(&uri, &config);
        assert!(header.to_str().unwrap().contains("no-cache"));
    }
}
