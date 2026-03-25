//! 认证流程集成测试
//!
//! 覆盖真实验证码登录、JWT 访问与 `/api/bindings` 权限边界。

mod support;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use support::{
    app_factory::create_test_app,
    auth_fixture::{
        admin_qq_number, get_with_bearer, login_with_seeded_code, post_json, read_json,
        unique_qq_number, UserInfo,
    },
};

#[tokio::test]
async fn admin_login_flow_returns_admin_profile() {
    let test_app = create_test_app().await;
    let login =
        login_with_seeded_code(&test_app.app, &test_app.state, &admin_qq_number(), "123456").await;

    assert_eq!(login.token_type, "Bearer");
    assert!(!login.access_token.is_empty());
    assert!(!login.refresh_token.is_empty());
    assert!(login.expires_in > 0);
    assert_eq!(login.user.role, "admin");
    assert!(login.user.is_active);
    assert_eq!(login.user.qq_number, admin_qq_number());

    let response = get_with_bearer(&test_app.app, "/api/auth/me", &login.access_token).await;
    assert_eq!(response.status(), StatusCode::OK);

    let current_user: UserInfo = read_json(response).await;
    assert_eq!(current_user.id, login.user.id);
    assert_eq!(current_user.qq_number, login.user.qq_number);
    assert_eq!(current_user.role, "admin");
}

#[tokio::test]
async fn admin_bindings_endpoint_returns_stable_empty_array() {
    let test_app = create_test_app().await;
    let login =
        login_with_seeded_code(&test_app.app, &test_app.state, &admin_qq_number(), "123456").await;

    let response = get_with_bearer(&test_app.app, "/api/bindings", &login.access_token).await;
    assert_eq!(response.status(), StatusCode::OK);

    let bindings: Vec<Value> = read_json(response).await;
    assert!(
        bindings.is_empty(),
        "管理员当前应返回稳定的空数组，而不是不确定状态"
    );
}

#[tokio::test]
async fn invalid_verification_code_is_rejected() {
    let test_app = create_test_app().await;
    let qq_number = unique_qq_number();

    let response = post_json(
        &test_app.app,
        "/api/auth/verify-and-login",
        serde_json::json!({
            "qq_number": qq_number,
            "code": "123456",
        }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body: Value = read_json(response).await;
    assert!(
        body.to_string().contains("验证码"),
        "登录失败响应应保留验证码错误语义"
    );
}

#[tokio::test]
async fn missing_bearer_prefix_is_rejected() {
    let test_app = create_test_app().await;
    let login =
        login_with_seeded_code(&test_app.app, &test_app.state, &admin_qq_number(), "123456").await;

    let request = Request::builder()
        .uri("/api/auth/me")
        .header(header::AUTHORIZATION, login.access_token)
        .body(Body::empty())
        .expect("构造请求失败");

    let response = test_app
        .app
        .clone()
        .oneshot(request)
        .await
        .expect("请求执行失败");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn user_login_can_access_me_and_bindings() {
    let test_app = create_test_app().await;
    let qq_number = unique_qq_number();
    let login = login_with_seeded_code(&test_app.app, &test_app.state, &qq_number, "654321").await;

    assert_eq!(login.user.role, "user");
    assert_eq!(login.user.qq_number, qq_number);

    let profile_response =
        get_with_bearer(&test_app.app, "/api/auth/me", &login.access_token).await;
    assert_eq!(profile_response.status(), StatusCode::OK);

    let current_user: UserInfo = read_json(profile_response).await;
    assert_eq!(current_user.id, login.user.id);
    assert_eq!(current_user.role, "user");
    assert_eq!(current_user.qq_number, qq_number);

    let bindings_response =
        get_with_bearer(&test_app.app, "/api/bindings", &login.access_token).await;
    assert_eq!(bindings_response.status(), StatusCode::OK);

    let bindings: Vec<Value> = read_json(bindings_response).await;
    assert!(
        bindings.is_empty(),
        "新创建的普通用户在未绑定房间时应返回空数组"
    );
}
