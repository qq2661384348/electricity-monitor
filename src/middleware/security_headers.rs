//! 统一追加运行时安全响应头。

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};

use crate::config::AppConfig;

const CSP_CONNECT_SRC_PREFIX: &str = "default-src 'self'; base-uri 'self'; frame-ancestors 'none'; object-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.cn; font-src 'self' https://fonts.gstatic.cn data:; img-src 'self' data: blob:; connect-src 'self'";
const DEFAULT_CAPTCHA_ORIGIN: &str = "https://v2.xxapi.cn";
const PERMISSIONS_POLICY_VALUE: &str = "geolocation=(), camera=(), microphone=()";

pub async fn apply(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    let content_security_policy = content_security_policy();

    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(PERMISSIONS_POLICY_VALUE),
    );
    headers.insert(
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        HeaderName::from_static("cross-origin-resource-policy"),
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        header_value_from_string(content_security_policy),
    );

    response
}

pub fn required_headers() -> Vec<(HeaderName, HeaderValue)> {
    let content_security_policy = content_security_policy();

    vec![
        (
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ),
        (
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ),
        (
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ),
        (
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static(PERMISSIONS_POLICY_VALUE),
        ),
        (
            HeaderName::from_static("cross-origin-opener-policy"),
            HeaderValue::from_static("same-origin"),
        ),
        (
            HeaderName::from_static("cross-origin-resource-policy"),
            HeaderValue::from_static("same-origin"),
        ),
        (
            HeaderName::from_static("content-security-policy"),
            header_value_from_string(content_security_policy),
        ),
    ]
}

fn content_security_policy() -> String {
    content_security_policy_for_captcha_api_url(&AppConfig::global().captcha.api_url)
}

fn content_security_policy_for_captcha_api_url(api_url: &str) -> String {
    let captcha_origin =
        captcha_origin_from_api_url(api_url).unwrap_or_else(|| DEFAULT_CAPTCHA_ORIGIN.to_string());
    format!("{CSP_CONNECT_SRC_PREFIX} {captcha_origin}")
}

fn header_value_from_string(value: String) -> HeaderValue {
    HeaderValue::from_str(&value).unwrap_or_else(|error| {
        tracing::error!(error = %error, "构造 Content-Security-Policy 响应头失败，回退到默认验证码源");
        HeaderValue::from_static(
            "default-src 'self'; base-uri 'self'; frame-ancestors 'none'; object-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.cn; font-src 'self' https://fonts.gstatic.cn data:; img-src 'self' data: blob:; connect-src 'self' https://v2.xxapi.cn",
        )
    })
}

fn captcha_origin_from_api_url(api_url: &str) -> Option<String> {
    let trimmed = api_url.trim();
    let (scheme, rest) = trimmed
        .strip_prefix("https://")
        .map(|rest| ("https", rest))
        .or_else(|| trimmed.strip_prefix("http://").map(|rest| ("http", rest)))?;
    let authority = rest
        .split(['/', '?', '#'])
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())?;

    if authority
        .chars()
        .any(|ch| ch.is_ascii_whitespace() || matches!(ch, '\'' | '"' | ';' | '\\'))
        || authority.contains('@')
    {
        return None;
    }

    Some(format!("{scheme}://{authority}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csp_uses_origin_from_configured_captcha_api_url() {
        assert_eq!(
            content_security_policy_for_captcha_api_url(
                "https://captcha.example.com:8443/api/captcha?type=math",
            ),
            "default-src 'self'; base-uri 'self'; frame-ancestors 'none'; object-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.cn; font-src 'self' https://fonts.gstatic.cn data:; img-src 'self' data: blob:; connect-src 'self' https://captcha.example.com:8443"
        );
    }

    #[test]
    fn csp_rejects_unsafe_captcha_origin_parts() {
        assert_eq!(
            content_security_policy_for_captcha_api_url("https://evil.example.com; script-src *"),
            "default-src 'self'; base-uri 'self'; frame-ancestors 'none'; object-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.cn; font-src 'self' https://fonts.gstatic.cn data:; img-src 'self' data: blob:; connect-src 'self' https://v2.xxapi.cn"
        );
    }
}
