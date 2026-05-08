#[path = "../support/mod.rs"]
mod support;

use std::{
    net::SocketAddr,
    sync::{Arc, OnceLock},
};

use axum::{
    body::{to_bytes, Body},
    extract::ConnectInfo,
    extract::State,
    http::{header, Request, StatusCode},
    routing::post,
    Json, Router,
};
use redis::AsyncCommands;
use serde_json::Value;
use tokio::sync::Mutex;
use tower::ServiceExt;

use electricity_monitor_backend::infrastructure::email::{
    EmailDelivery, EmailError, RenderedEmail, Result as EmailResult,
};

use support::app_factory::{create_test_app, create_test_app_with_email_sender, test_config};

struct MockQqServer {
    base_url: String,
}

static QQ_SERVER: tokio::sync::OnceCell<MockQqServer> = tokio::sync::OnceCell::const_new();
static QQ_ENV_INIT: OnceLock<()> = OnceLock::new();
static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Default)]
struct MockEmailSender {
    verification_codes: Mutex<Vec<(String, String, String)>>,
    rendered_emails: Mutex<Vec<(String, RenderedEmail)>>,
}

#[async_trait::async_trait]
impl EmailDelivery for MockEmailSender {
    async fn send_verification_code(
        &self,
        to_email: &str,
        code: &str,
        scene: &str,
    ) -> EmailResult<()> {
        self.verification_codes.lock().await.push((
            to_email.to_string(),
            code.to_string(),
            scene.to_string(),
        ));
        Ok(())
    }

    async fn send_rendered_email(
        &self,
        to_email: &str,
        rendered: &RenderedEmail,
    ) -> EmailResult<()> {
        if to_email.trim().is_empty() {
            return Err(EmailError::Address(to_email.to_string()));
        }

        self.rendered_emails
            .lock()
            .await
            .push((to_email.to_string(), rendered.clone()));
        Ok(())
    }
}

#[derive(Clone)]
struct MockState;

async fn qq_mock_handler(
    State(_state): State<MockState>,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let user_id = payload["user_id"].as_str().unwrap_or_default();

    if user_id.starts_with("1999999999999") {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "failed",
                "retcode": 200,
                "message": "无法获取用户信息"
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "retcode": 0,
            "data": {
                "message_id": 1
            }
        })),
    )
}

async fn ensure_mock_qq_server() -> &'static MockQqServer {
    let server = QQ_SERVER
        .get_or_init(|| async {
            let listener =
                std::net::TcpListener::bind("127.0.0.1:0").expect("启动 QQ mock server 失败");
            listener
                .set_nonblocking(true)
                .expect("设置 QQ mock server 非阻塞失败");
            let addr = listener.local_addr().expect("获取 QQ mock server 地址失败");

            let router = Router::new()
                .route("/send_private_msg", post(qq_mock_handler))
                .with_state(MockState);

            std::thread::spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("创建 QQ mock server runtime 失败");
                runtime.block_on(async move {
                    let listener = tokio::net::TcpListener::from_std(listener)
                        .expect("接管 QQ mock server listener 失败");
                    axum::serve(listener, router)
                        .await
                        .expect("运行 QQ mock server 失败");
                });
            });

            MockQqServer {
                base_url: format!("http://{addr}/send_private_msg"),
            }
        })
        .await;

    QQ_ENV_INIT.get_or_init(|| {
        std::env::set_var("APP__QQ_BOT__API_URL", &server.base_url);
        std::env::set_var("APP__QQ_BOT__BEARER_TOKEN", "mock-token");
        std::env::set_var("APP_ENV", "development");
    });

    server
}

fn test_mutex() -> &'static Mutex<()> {
    TEST_MUTEX.get_or_init(|| Mutex::new(()))
}

async fn seed_captcha_token(state: &electricity_monitor_backend::state::AppState, token: &str) {
    let mut conn = state.redis_pool.get().await.expect("获取 Redis 连接失败");
    conn.set_ex::<_, _, ()>(&format!("captcha:token:{token}"), "valid", 60)
        .await
        .expect("写入 captcha token 失败");
}

async fn clear_auth_send_code_rate_limits(state: &electricity_monitor_backend::state::AppState) {
    let mut conn = state.redis_pool.get().await.expect("获取 Redis 连接失败");
    let keys: Vec<String> = conn
        .keys("ratelimit:auth-send-code*")
        .await
        .expect("扫描验证码限流键失败");

    if !keys.is_empty() {
        let _: () = conn.del(keys).await.expect("清理验证码限流键失败");
    }
}

async fn send_code_request(
    app: &Router,
    state: &electricity_monitor_backend::state::AppState,
    qq_number: &str,
) -> axum::response::Response {
    // 发送 QQ 验证码会触达机器人服务，必须先消费后端签发的一次性 captcha token。
    // 测试直接种入 token，避免依赖第三方验证码服务，同时保持真实 send-code 契约。
    let captcha_token = format!("captcha-token-{qq_number}");
    seed_captcha_token(state, &captcha_token).await;

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/send-verification-code")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "qq_number": qq_number,
                "captcha_token": captcha_token,
            })
            .to_string(),
        ))
        .expect("构造发送验证码请求失败");

    app.clone().oneshot(request).await.expect("请求执行失败")
}

async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("读取响应体失败");
    serde_json::from_slice(&body).expect("响应体应为 JSON")
}

async fn redis_code_for(
    qq_number: &str,
    state: &electricity_monitor_backend::state::AppState,
) -> Option<String> {
    redis_code_for_identity("qq", qq_number, state).await
}

async fn redis_code_for_identity(
    login_provider: &str,
    identifier: &str,
    state: &electricity_monitor_backend::state::AppState,
) -> Option<String> {
    let key = test_config()
        .verification
        .redis_key_for(login_provider, identifier);
    let mut conn = state.redis_pool.get().await.expect("获取 Redis 连接失败");
    conn.get(&key).await.expect("读取 Redis 验证码失败")
}

async fn send_email_code_with_peer_and_forwarded_for(
    app: &Router,
    state: &electricity_monitor_backend::state::AppState,
    email: &str,
    captcha_token: &str,
    peer_addr: SocketAddr,
    forwarded_for: &str,
) -> axum::response::Response {
    seed_captcha_token(state, captcha_token).await;

    let mut request = Request::builder()
        .method("POST")
        .uri("/api/auth/send-verification-code")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-for", forwarded_for)
        .body(Body::from(
            serde_json::json!({
                "login_mode": "email",
                "identifier": email,
                "captcha_token": captcha_token,
            })
            .to_string(),
        ))
        .expect("构造邮箱发送验证码请求失败");
    request.extensions_mut().insert(ConnectInfo(peer_addr));

    app.clone().oneshot(request).await.expect("请求执行失败")
}

#[tokio::test]
async fn send_verification_code_mocked_qq_api_covers_success_and_user_not_friend() {
    let _guard = test_mutex().lock().await;
    ensure_mock_qq_server().await;
    let test_app = create_test_app().await;
    clear_auth_send_code_rate_limits(&test_app.state).await;
    let unique_suffix = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("系统时间异常")
        .as_millis()
        % 1_000_000) as u64;
    let success_qq_number = format!("188{unique_suffix}");

    let response = send_code_request(&test_app.app, &test_app.state, &success_qq_number).await;
    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected response body: {body}");

    assert_eq!(body["message"], "验证码已发送");
    assert_eq!(body["qq_number"], success_qq_number);

    let stored_code = redis_code_for(&success_qq_number, &test_app.state)
        .await
        .expect("发送成功后应在 Redis 写入验证码");
    assert_eq!(stored_code.len(), 6);
    assert!(stored_code.chars().all(|c| c.is_ascii_digit()));
    let error_qq_number = format!("1999999999999{unique_suffix}");

    let response = send_code_request(&test_app.app, &test_app.state, &error_qq_number).await;
    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "unexpected response body: {body}"
    );

    assert_eq!(body["error"], "USER_NOT_FRIEND");
    assert_eq!(body["qq_number"], error_qq_number);

    let stored_code = redis_code_for(&error_qq_number, &test_app.state).await;
    assert!(
        stored_code.is_none(),
        "用户未加好友时不应把验证码写入 Redis"
    );
}

#[tokio::test]
async fn send_email_verification_code_uses_captcha_and_mocked_mail_sender() {
    let _guard = test_mutex().lock().await;
    ensure_mock_qq_server().await;
    let email_sender = Arc::new(MockEmailSender::default());
    let test_app =
        create_test_app_with_email_sender(Some(email_sender.clone() as Arc<dyn EmailDelivery>))
            .await;
    clear_auth_send_code_rate_limits(&test_app.state).await;
    let email = "Student.Login@Example.COM";
    let normalized_email = "student.login@example.com";
    let captcha_token = "captcha-token-email-login";
    seed_captcha_token(&test_app.state, captcha_token).await;

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/send-verification-code")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "login_mode": "email",
                "identifier": email,
                "captcha_token": captcha_token,
            })
            .to_string(),
        ))
        .expect("构造邮箱发送验证码请求失败");

    let response = test_app
        .app
        .clone()
        .oneshot(request)
        .await
        .expect("请求执行失败");
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["message"], "验证码已发送");
    assert_eq!(body["login_mode"], "email");
    assert_eq!(body["identifier"], normalized_email);
    assert_eq!(body["email"], normalized_email);

    let sent_codes = email_sender.verification_codes.lock().await;
    assert_eq!(sent_codes.len(), 1);
    assert_eq!(sent_codes[0].0, normalized_email);
    assert_eq!(sent_codes[0].2, "login");
    assert_eq!(
        sent_codes[0].1.len(),
        test_config().verification.code_length
    );
    drop(sent_codes);

    let stored_code = redis_code_for_identity("email", normalized_email, &test_app.state)
        .await
        .expect("邮箱验证码发送成功后应写入 Redis");
    let sent_codes = email_sender.verification_codes.lock().await;
    assert_eq!(stored_code, sent_codes[0].1);
}

#[tokio::test]
async fn send_verification_code_rate_limits_same_destination_before_delivery() {
    let _guard = test_mutex().lock().await;
    ensure_mock_qq_server().await;
    let email_sender = Arc::new(MockEmailSender::default());
    let test_app =
        create_test_app_with_email_sender(Some(email_sender.clone() as Arc<dyn EmailDelivery>))
            .await;
    clear_auth_send_code_rate_limits(&test_app.state).await;
    let email = format!(
        "limited-{}@example.com",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("系统时间异常")
            .as_millis()
    );

    for attempt in 0..3 {
        let captcha_token = format!("captcha-token-rate-limit-{attempt}");
        seed_captcha_token(&test_app.state, &captcha_token).await;

        let request = Request::builder()
            .method("POST")
            .uri("/api/auth/send-verification-code")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::json!({
                    "login_mode": "email",
                    "identifier": email,
                    "captcha_token": captcha_token,
                })
                .to_string(),
            ))
            .expect("构造邮箱发送验证码请求失败");

        let response = test_app
            .app
            .clone()
            .oneshot(request)
            .await
            .expect("请求执行失败");
        assert_eq!(response.status(), StatusCode::OK);
    }

    let captcha_token = "captcha-token-rate-limit-denied";
    seed_captcha_token(&test_app.state, captcha_token).await;
    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/send-verification-code")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "login_mode": "email",
                "identifier": email,
                "captcha_token": captcha_token,
            })
            .to_string(),
        ))
        .expect("构造邮箱发送验证码请求失败");

    let response = test_app
        .app
        .clone()
        .oneshot(request)
        .await
        .expect("请求执行失败");
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    let sent_codes = email_sender.verification_codes.lock().await;
    assert_eq!(
        sent_codes.len(),
        3,
        "目标限流命中后不应继续触达 SMTP 发送器"
    );
}

#[tokio::test]
async fn send_verification_code_client_limit_uses_peer_address_not_forwarded_headers() {
    let _guard = test_mutex().lock().await;
    ensure_mock_qq_server().await;
    let email_sender = Arc::new(MockEmailSender::default());
    let test_app =
        create_test_app_with_email_sender(Some(email_sender.clone() as Arc<dyn EmailDelivery>))
            .await;
    clear_auth_send_code_rate_limits(&test_app.state).await;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("系统时间异常")
        .as_millis();
    let peer_addr: SocketAddr = "203.0.113.10:52100".parse().expect("测试 peer 地址应合法");

    for attempt in 0..10 {
        let email = format!("peer-limit-{timestamp}-{attempt}@example.com");
        let captcha_token = format!("captcha-token-peer-limit-{timestamp}-{attempt}");
        let response = send_email_code_with_peer_and_forwarded_for(
            &test_app.app,
            &test_app.state,
            &email,
            &captcha_token,
            peer_addr,
            &format!("198.51.100.{attempt}"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    let denied_email = format!("peer-limit-{timestamp}-denied@example.com");
    let denied_token = format!("captcha-token-peer-limit-{timestamp}-denied");
    let denied_response = send_email_code_with_peer_and_forwarded_for(
        &test_app.app,
        &test_app.state,
        &denied_email,
        &denied_token,
        peer_addr,
        "198.51.100.200",
    )
    .await;

    assert_eq!(denied_response.status(), StatusCode::TOO_MANY_REQUESTS);
    let sent_codes = email_sender.verification_codes.lock().await;
    assert_eq!(
        sent_codes.len(),
        10,
        "客户端限流命中后不应继续触达 SMTP 发送器"
    );
}
