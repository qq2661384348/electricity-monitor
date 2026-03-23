//! 日志中间件

use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

/// 将配置字符串转换为 tracing::Level
fn parse_level(level: &str) -> Level {
    match level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" | "warning" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    }
}

/// 创建日志跟踪层
///
/// # 参数
/// - `level`: 日志级别字符串（trace/debug/info/warn/error）
pub fn create_trace_layer(
    level: &str,
) -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    let level = parse_level(level);
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(level))
        .on_response(DefaultOnResponse::new().level(level))
}
