//! 认证流程集成测试
//!
//! 覆盖真实验证码登录、JWT 访问与 `/api/bindings` 权限边界。

#[path = "../support/mod.rs"]
mod support;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use electricity_monitor_backend::{
    domain::{
        models::Room,
        services::{RoomData, RoomPathTree},
    },
    state::AppState,
    utils::hash::calculate_roompath_hash,
};

use support::{
    app_factory::create_test_app,
    auth_fixture::{
        admin_qq_number, delete_with_bearer, get_with_bearer, login_with_seeded_code, post_json,
        post_json_with_bearer, post_with_cookie, put_json_with_bearer, raw_refresh_token,
        read_json, unique_qq_number, UserInfo,
    },
    seed::{delete_room, seed_room},
};

async fn rebuild_path_tree_for_room(state: &AppState, room: &Room) {
    state
        .update_path_tree(RoomPathTree::build_from_rooms(&[RoomData {
            roomid: room.roomid,
            roompaths: vec![room.primary_roompath.clone()],
            primary_roompath: room.primary_roompath.clone(),
            path_count: 1,
        }]))
        .await;
}

#[tokio::test]
async fn admin_login_flow_returns_admin_profile() {
    let test_app = create_test_app().await;
    let login =
        login_with_seeded_code(&test_app.app, &test_app.state, &admin_qq_number(), "123456").await;

    assert_eq!(login.token_type, "Bearer");
    assert!(!login.access_token.is_empty());
    assert!(!login.refresh_cookie.is_empty());
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
async fn admin_bindings_endpoint_returns_authenticated_array() {
    let test_app = create_test_app().await;
    let login =
        login_with_seeded_code(&test_app.app, &test_app.state, &admin_qq_number(), "123456").await;

    let response = get_with_bearer(&test_app.app, "/api/bindings", &login.access_token).await;
    assert_eq!(response.status(), StatusCode::OK);

    let _bindings: Vec<Value> = read_json(response).await;
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
async fn send_verification_code_rejects_invalid_captcha_token() {
    let test_app = create_test_app().await;
    let response = post_json(
        &test_app.app,
        "/api/auth/send-verification-code",
        serde_json::json!({
            "qq_number": unique_qq_number(),
            "captcha_token": "invalid-captcha-token",
        }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn send_verification_code_requires_captcha_token() {
    let test_app = create_test_app().await;
    let response = post_json(
        &test_app.app,
        "/api/auth/send-verification-code",
        serde_json::json!({
            "qq_number": unique_qq_number(),
        }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body: Value = read_json(response).await;
    assert!(
        body.to_string().contains("验证码token"),
        "缺少 captcha token 时应在调用 QQ 服务前拒绝"
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

#[tokio::test]
async fn refresh_token_returns_new_access_token_for_same_user() {
    let test_app = create_test_app().await;
    let qq_number = unique_qq_number();
    let login = login_with_seeded_code(&test_app.app, &test_app.state, &qq_number, "654321").await;

    let response =
        post_with_cookie(&test_app.app, "/api/auth/refresh", &login.refresh_cookie).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers().contains_key(header::SET_COOKIE),
        "refresh 成功后应轮换 refresh cookie"
    );

    let refreshed: Value = read_json(response).await;
    assert_eq!(refreshed["user"]["qq_number"], qq_number);
    assert_eq!(refreshed["user"]["role"], "user");
    assert_eq!(refreshed["token_type"], "Bearer");
    assert!(refreshed.get("refresh_token").is_none());
    assert_ne!(
        refreshed["access_token"],
        Value::String(String::new()),
        "刷新后应返回新的 access token"
    );
}

#[tokio::test]
async fn refresh_token_cookie_cannot_access_protected_routes_as_bearer() {
    let test_app = create_test_app().await;
    let login = login_with_seeded_code(
        &test_app.app,
        &test_app.state,
        &unique_qq_number(),
        "654321",
    )
    .await;
    let refresh_token = raw_refresh_token(&login.refresh_cookie);

    let response = get_with_bearer(&test_app.app, "/api/auth/me", &refresh_token).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn access_token_cannot_refresh_session() {
    let test_app = create_test_app().await;
    let login = login_with_seeded_code(
        &test_app.app,
        &test_app.state,
        &unique_qq_number(),
        "654321",
    )
    .await;

    let response = post_with_cookie(
        &test_app.app,
        "/api/auth/refresh",
        &format!("refresh_token={}", login.access_token),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_clears_refresh_cookie() {
    let test_app = create_test_app().await;
    let login = login_with_seeded_code(
        &test_app.app,
        &test_app.state,
        &unique_qq_number(),
        "654321",
    )
    .await;

    let response = post_with_cookie(&test_app.app, "/api/auth/logout", &login.refresh_cookie).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let cleared_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .expect("logout 响应应返回清理 cookie 头");
    assert!(cleared_cookie.contains("refresh_token="));
    assert!(cleared_cookie.contains("Max-Age=0"));
}

#[tokio::test]
async fn user_can_complete_binding_crud_flow() {
    let test_app = create_test_app().await;
    let room = seed_room(&test_app.state).await;
    let qq_number = unique_qq_number();
    let login = login_with_seeded_code(&test_app.app, &test_app.state, &qq_number, "654321").await;

    let create_response = post_json_with_bearer(
        &test_app.app,
        "/api/bindings",
        &login.access_token,
        serde_json::json!({
            "roomid": room.roomid,
            "notification_enabled": false,
        }),
    )
    .await;

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created_binding: Value = read_json(create_response).await;
    let binding_id = created_binding["id"]
        .as_str()
        .expect("绑定 ID 应为字符串")
        .to_string();
    assert_eq!(created_binding["roomid"], room.roomid);
    assert_eq!(created_binding["notification_enabled"], false);

    let list_response = get_with_bearer(&test_app.app, "/api/bindings", &login.access_token).await;
    assert_eq!(list_response.status(), StatusCode::OK);
    let bindings: Vec<Value> = read_json(list_response).await;
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0]["room"]["roomid"], room.roomid);

    let detail_response = get_with_bearer(
        &test_app.app,
        &format!("/api/bindings/{binding_id}"),
        &login.access_token,
    )
    .await;
    assert_eq!(detail_response.status(), StatusCode::OK);

    let update_response = put_json_with_bearer(
        &test_app.app,
        &format!("/api/bindings/{binding_id}/notification"),
        &login.access_token,
        serde_json::json!({ "notification_enabled": true }),
    )
    .await;
    assert_eq!(update_response.status(), StatusCode::OK);
    let updated_binding: Value = read_json(update_response).await;
    assert_eq!(updated_binding["notification_enabled"], true);

    let delete_response = delete_with_bearer(
        &test_app.app,
        &format!("/api/bindings/{binding_id}"),
        &login.access_token,
    )
    .await;
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let list_after_delete =
        get_with_bearer(&test_app.app, "/api/bindings", &login.access_token).await;
    let bindings_after_delete: Vec<Value> = read_json(list_after_delete).await;
    assert!(bindings_after_delete.is_empty());

    delete_room(&test_app.state, room.id).await;
}

#[tokio::test]
async fn user_cannot_access_other_users_binding() {
    let test_app = create_test_app().await;
    let room = seed_room(&test_app.state).await;
    let owner_login = login_with_seeded_code(
        &test_app.app,
        &test_app.state,
        &unique_qq_number(),
        "111111",
    )
    .await;
    let other_login = login_with_seeded_code(
        &test_app.app,
        &test_app.state,
        &unique_qq_number(),
        "222222",
    )
    .await;

    let create_response = post_json_with_bearer(
        &test_app.app,
        "/api/bindings",
        &owner_login.access_token,
        serde_json::json!({
            "roomid": room.roomid,
            "notification_enabled": false,
        }),
    )
    .await;
    let created_binding: Value = read_json(create_response).await;
    let binding_id = created_binding["id"]
        .as_str()
        .expect("绑定 ID 应为字符串")
        .to_string();

    let detail_response = get_with_bearer(
        &test_app.app,
        &format!("/api/bindings/{binding_id}"),
        &other_login.access_token,
    )
    .await;
    assert_eq!(detail_response.status(), StatusCode::UNAUTHORIZED);

    let _ = delete_with_bearer(
        &test_app.app,
        &format!("/api/bindings/{binding_id}"),
        &owner_login.access_token,
    )
    .await;

    delete_room(&test_app.state, room.id).await;
}

#[tokio::test]
async fn user_needs_binding_before_reading_room_path_details() {
    let test_app = create_test_app().await;
    let room = seed_room(&test_app.state).await;
    rebuild_path_tree_for_room(&test_app.state, &room).await;

    let login = login_with_seeded_code(
        &test_app.app,
        &test_app.state,
        &unique_qq_number(),
        "333333",
    )
    .await;

    let (parent_path, leaf_name) = room
        .primary_roompath
        .rsplit_once('/')
        .expect("测试房间路径应包含父级与房间名");
    let path_tree_response = get_with_bearer(
        &test_app.app,
        &format!(
            "/api/rooms/path-tree?parent={}",
            urlencoding::encode(parent_path)
        ),
        &login.access_token,
    )
    .await;
    assert_eq!(path_tree_response.status(), StatusCode::OK);
    let path_tree: Value = read_json(path_tree_response).await;
    let leaf = path_tree["children"]
        .as_array()
        .and_then(|children| {
            children
                .iter()
                .find(|child| child["name"] == Value::String(leaf_name.to_string()))
        })
        .expect("路径树叶子节点应存在");
    assert_eq!(leaf["roomid"].as_i64(), Some(room.roomid as i64));
    assert!(
        leaf.get("electricity_fee").is_none(),
        "绑定入口只能返回 roomid，不应提前泄露电费详情"
    );

    let encoded_path = urlencoding::encode(&room.primary_roompath);
    let unbound_by_path = get_with_bearer(
        &test_app.app,
        &format!("/api/rooms/by-path?path={encoded_path}"),
        &login.access_token,
    )
    .await;
    assert_eq!(unbound_by_path.status(), StatusCode::FORBIDDEN);

    let hash = calculate_roompath_hash(&room.primary_roompath);
    let unbound_by_hash = get_with_bearer(
        &test_app.app,
        &format!("/api/rooms/by-hash?hash={hash}&path={encoded_path}"),
        &login.access_token,
    )
    .await;
    assert_eq!(unbound_by_hash.status(), StatusCode::FORBIDDEN);

    let create_response = post_json_with_bearer(
        &test_app.app,
        "/api/bindings",
        &login.access_token,
        serde_json::json!({
            "roomid": room.roomid,
            "notification_enabled": false,
        }),
    )
    .await;
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created_binding: Value = read_json(create_response).await;
    let binding_id = created_binding["id"]
        .as_str()
        .expect("绑定 ID 应为字符串")
        .to_string();

    let bound_by_path = get_with_bearer(
        &test_app.app,
        &format!("/api/rooms/by-path?path={encoded_path}"),
        &login.access_token,
    )
    .await;
    assert_eq!(bound_by_path.status(), StatusCode::OK);
    let room_detail: Value = read_json(bound_by_path).await;
    assert_eq!(room_detail["roomid"].as_i64(), Some(room.roomid as i64));
    assert_eq!(room_detail["electricity_fee"].as_f64(), Some(25.0));

    let _ = delete_with_bearer(
        &test_app.app,
        &format!("/api/bindings/{binding_id}"),
        &login.access_token,
    )
    .await;
    delete_room(&test_app.state, room.id).await;
}

#[tokio::test]
async fn admin_can_create_and_list_own_binding() {
    let test_app = create_test_app().await;
    let room = seed_room(&test_app.state).await;
    let admin_login =
        login_with_seeded_code(&test_app.app, &test_app.state, &admin_qq_number(), "123456").await;

    let create_response = post_json_with_bearer(
        &test_app.app,
        "/api/bindings",
        &admin_login.access_token,
        serde_json::json!({
            "roomid": room.roomid,
            "notification_enabled": false,
        }),
    )
    .await;

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created_binding: Value = read_json(create_response).await;
    let binding_id = created_binding["id"]
        .as_str()
        .expect("绑定 ID 应为字符串")
        .to_string();
    assert_eq!(created_binding["roomid"], room.roomid);

    let list_response =
        get_with_bearer(&test_app.app, "/api/bindings", &admin_login.access_token).await;
    assert_eq!(list_response.status(), StatusCode::OK);
    let bindings: Vec<Value> = read_json(list_response).await;
    assert!(
        bindings
            .iter()
            .any(|binding| binding["id"].as_str() == Some(binding_id.as_str())),
        "管理员创建个人绑定后应能在自己的绑定列表中看到它"
    );

    let delete_response = delete_with_bearer(
        &test_app.app,
        &format!("/api/bindings/{binding_id}"),
        &admin_login.access_token,
    )
    .await;
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    delete_room(&test_app.state, room.id).await;
}
