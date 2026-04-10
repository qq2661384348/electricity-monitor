use std::{
    sync::atomic::{AtomicU64, Ordering},
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
    response::Response,
    Router,
};
use electricity_monitor_backend::state::AppState;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Deserialize};
use tokio::sync::Mutex;
use tower::ServiceExt;

use super::app_factory::test_config;

static QQ_COUNTER: AtomicU64 = AtomicU64::new(0);
static ADMIN_LOGIN_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserInfo,
    #[serde(skip)]
    pub refresh_cookie: String,
}

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub qq_number: String,
    pub role: String,
    pub is_active: bool,
}

pub fn admin_qq_number() -> String {
    test_config().admin.default_qq_number.clone()
}

pub fn unique_qq_number() -> String {
    let counter = QQ_COUNTER.fetch_add(1, Ordering::Relaxed);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("系统时间异常")
        .as_millis() as u64;

    format!("9{:013}", (now + counter) % 10_000_000_000_000)
}

pub async fn login_with_seeded_code(
    app: &Router,
    state: &AppState,
    qq_number: &str,
    code: &str,
) -> LoginResponse {
    if qq_number == test_config().admin.default_qq_number {
        let _guard = admin_login_lock().lock().await;
        return login_with_seeded_code_inner(app, state, qq_number, code).await;
    }

    login_with_seeded_code_inner(app, state, qq_number, code).await
}

async fn login_with_seeded_code_inner(
    app: &Router,
    state: &AppState,
    qq_number: &str,
    code: &str,
) -> LoginResponse {
    seed_verification_code(state, qq_number, code).await;

    let response = post_json(
        app,
        "/api/auth/verify-and-login",
        serde_json::json!({
            "qq_number": qq_number,
            "code": code,
        }),
    )
    .await;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "真实登录链路应该返回 200"
    );

    let refresh_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(cookie_header_value)
        .expect("登录响应必须返回 refresh cookie")
        .to_string();

    let mut login: LoginResponse = read_json(response).await;
    login.refresh_cookie = refresh_cookie;
    login
}

pub async fn post_json(app: &Router, uri: &str, payload: serde_json::Value) -> Response {
    json_request(app, "POST", uri, None, payload).await
}

pub async fn post_with_cookie(app: &Router, uri: &str, cookie: &str) -> Response {
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .expect("构造 Cookie 请求失败");

    app.clone().oneshot(request).await.expect("请求执行失败")
}

pub async fn post_json_with_bearer(
    app: &Router,
    uri: &str,
    token: &str,
    payload: serde_json::Value,
) -> Response {
    json_request(app, "POST", uri, Some(token), payload).await
}

pub async fn put_json_with_bearer(
    app: &Router,
    uri: &str,
    token: &str,
    payload: serde_json::Value,
) -> Response {
    json_request(app, "PUT", uri, Some(token), payload).await
}

pub async fn delete_with_bearer(app: &Router, uri: &str, token: &str) -> Response {
    let request = Request::builder()
        .method("DELETE")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("构造删除请求失败");

    app.clone().oneshot(request).await.expect("请求执行失败")
}

async fn json_request(
    app: &Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    payload: serde_json::Value,
) -> Response {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");

    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {}", token));
    }

    let request = request
        .body(Body::from(payload.to_string()))
        .expect("构造 JSON 请求失败");

    app.clone().oneshot(request).await.expect("请求执行失败")
}

pub fn raw_refresh_token(cookie: &str) -> String {
    cookie
        .strip_prefix("refresh_token=")
        .and_then(|value| value.split(';').next())
        .expect("refresh cookie 应包含 refresh_token")
        .to_string()
}

fn cookie_header_value(set_cookie: &str) -> Option<&str> {
    set_cookie
        .split(';')
        .next()
        .filter(|segment| segment.starts_with("refresh_token="))
}

pub async fn get_with_bearer(app: &Router, uri: &str, token: &str) -> Response {
    let request = Request::builder()
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("构造认证请求失败");

    app.clone().oneshot(request).await.expect("请求执行失败")
}

pub async fn read_json<T: DeserializeOwned>(response: Response) -> T {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("读取响应体失败");

    serde_json::from_slice(&body).expect("响应体应为有效 JSON")
}

async fn seed_verification_code(state: &AppState, qq_number: &str, code: &str) {
    let config = test_config();
    let key = config.verification.redis_key(qq_number);
    let mut conn = state.redis_pool.get().await.expect("获取 Redis 连接失败");

    conn.set_ex::<_, _, ()>(&key, code, config.verification.expire_seconds)
        .await
        .expect("写入验证码失败");
}

fn admin_login_lock() -> &'static Mutex<()> {
    ADMIN_LOGIN_LOCK.get_or_init(|| Mutex::new(()))
}
