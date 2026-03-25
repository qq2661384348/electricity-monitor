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
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserInfo,
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

    read_json(response).await
}

pub async fn post_json(app: &Router, uri: &str, payload: serde_json::Value) -> Response {
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .expect("构造 JSON 请求失败");

    app.clone().oneshot(request).await.expect("请求执行失败")
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
