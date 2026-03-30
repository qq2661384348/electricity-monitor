#[path = "../support/mod.rs"]
mod support;

use std::sync::OnceLock;

use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{header, Request, StatusCode},
    routing::post,
    Json, Router,
};
use redis::AsyncCommands;
use serde_json::Value;
use tokio::{
    net::TcpListener,
    sync::{oneshot, Mutex},
};
use tower::ServiceExt;

use support::app_factory::{create_test_app, test_config};

struct MockQqServer {
    base_url: String,
}

static QQ_SERVER: tokio::sync::OnceCell<MockQqServer> = tokio::sync::OnceCell::const_new();
static QQ_ENV_INIT: OnceLock<()> = OnceLock::new();
static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Clone)]
struct MockState;

async fn qq_mock_handler(
    State(_state): State<MockState>,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let user_id = payload["user_id"].as_str().unwrap_or_default();

    if user_id == "1999999999999" {
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
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("启动 QQ mock server 失败");
            let addr = listener.local_addr().expect("获取 QQ mock server 地址失败");

            let router = Router::new()
                .route("/send_private_msg", post(qq_mock_handler))
                .with_state(MockState);

            let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
            tokio::spawn(async move {
                let _keepalive = shutdown_tx;
                axum::serve(listener, router)
                    .with_graceful_shutdown(async move {
                        let _ = shutdown_rx.await;
                    })
                    .await
                    .expect("运行 QQ mock server 失败");
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

async fn send_code_request(app: &Router, qq_number: &str) -> axum::response::Response {
    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/send-verification-code")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "qq_number": qq_number
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
    let key = test_config().verification.redis_key(qq_number);
    let mut conn = state.redis_pool.get().await.expect("获取 Redis 连接失败");
    conn.get(&key).await.expect("读取 Redis 验证码失败")
}

#[tokio::test]
async fn send_verification_code_mocked_qq_api_covers_success_and_user_not_friend() {
    let _guard = test_mutex().lock().await;
    ensure_mock_qq_server().await;
    let test_app = create_test_app().await;
    let success_qq_number = "1888888888888";

    let response = send_code_request(&test_app.app, success_qq_number).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["message"], "验证码已发送");
    assert_eq!(body["qq_number"], success_qq_number);

    let stored_code = redis_code_for(success_qq_number, &test_app.state)
        .await
        .expect("发送成功后应在 Redis 写入验证码");
    assert_eq!(stored_code.len(), 6);
    assert!(stored_code.chars().all(|c| c.is_ascii_digit()));
    let error_qq_number = "1999999999999";

    let response = send_code_request(&test_app.app, error_qq_number).await;
    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "unexpected response body: {body}"
    );

    assert_eq!(body["error"], "USER_NOT_FRIEND");
    assert_eq!(body["qq_number"], error_qq_number);

    let stored_code = redis_code_for(error_qq_number, &test_app.state).await;
    assert!(
        stored_code.is_none(),
        "用户未加好友时不应把验证码写入 Redis"
    );
}
