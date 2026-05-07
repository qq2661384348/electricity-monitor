#[path = "../support/mod.rs"]
mod support;

use axum::{
    body::{to_bytes, Body},
    http::{HeaderMap, Request, StatusCode},
};
use serde_json::Value;
use std::{fs, path::PathBuf};
use tower::ServiceExt;

use support::{
    app_factory::{create_test_app, test_config},
    smoke_contract::{load_smoke_targets, smoke_targets_path, SmokeTargets},
};

#[tokio::test]
async fn runtime_endpoints_from_smoke_contract_pass() {
    let test_app = create_test_app().await;
    let targets = load_smoke_targets();

    for endpoint in [
        &targets.health_endpoint,
        &targets.db_health_endpoint,
        &targets.static_entry,
    ] {
        let response = test_app
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(endpoint.as_str())
                    .body(Body::empty())
                    .expect("构造请求失败"),
            )
            .await
            .expect("请求执行失败");

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "smoke 契约中的运行时端点应全部可用: {endpoint}"
        );
        assert_required_headers(response.headers(), &targets, endpoint.as_str());
    }
}

#[tokio::test]
async fn public_config_exposes_only_non_sensitive_runtime_values() {
    let test_app = create_test_app().await;
    let targets = load_smoke_targets();
    let config = test_config();

    let response = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/public-config")
                .body(Body::empty())
                .expect("构造请求失败"),
        )
        .await
        .expect("请求执行失败");

    assert_eq!(response.status(), StatusCode::OK);
    assert_required_headers(response.headers(), &targets, "/api/public-config");

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("读取响应体失败");
    let body: Value = serde_json::from_slice(&body).expect("响应体应为 JSON");

    assert_eq!(
        body["notification"]["qq_bot_public_qq_number"].as_str(),
        Some(config.qq_bot.public_qq_number.as_str())
    );
    assert_eq!(
        body["notification"]["admin_qq_number"].as_str(),
        Some(config.admin.default_qq_number.as_str())
    );
    assert_eq!(
        body["verification"]["code_length"].as_u64(),
        Some(config.verification.code_length as u64)
    );
    assert_eq!(
        body["auth"]["email_login_enabled"].as_bool(),
        Some(config.email.is_delivery_configured())
    );
    assert!(
        body["auth"]["login_modes"]
            .as_array()
            .is_some_and(|modes| modes.iter().any(|mode| mode.as_str() == Some("qq"))),
        "公开配置必须至少声明 QQ 登录模式"
    );
    assert_eq!(
        body["captcha"]["captcha_type"].as_str(),
        Some(config.captcha.captcha_type.as_str())
    );
    assert!(
        body.get("qq_bot").is_none(),
        "公开配置不能暴露完整 qq_bot 配置或 bearer token"
    );
    assert!(body.get("email").is_none(), "公开配置不能暴露 SMTP 配置");
}

#[tokio::test]
async fn smoke_contract_tracks_release_artifact_files() {
    let targets = load_smoke_targets();

    assert!(
        smoke_targets_path().exists(),
        "release smoke 契约文件必须存在"
    );
    assert_eq!(
        targets.required_release_files,
        vec![
            "release-manifest.json".to_string(),
            "deploy-result.json".to_string(),
        ]
    );
    assert!(
        !targets.required_headers.is_empty(),
        "smoke 契约必须声明统一响应安全头"
    );
}

#[test]
fn release_packaging_splits_app_and_infra_artifacts() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workflow = fs::read_to_string(repo_root.join(".github/workflows/docker-build.yml"))
        .expect("应能读取 release workflow");
    let deploy_script =
        fs::read_to_string(repo_root.join("deploy/deploy.sh")).expect("应能读取 deploy.sh");

    assert!(
        workflow.contains("electricity-monitor-app-release-${{ inputs.git_tag }}"),
        "日常发布 artifact 必须只下载应用 release 包"
    );
    assert!(
        workflow.contains("electricity-monitor-infra-images-${{ inputs.git_tag }}"),
        "infra 镜像必须拆成独立 artifact，供首次部署或基础镜像变更时使用"
    );
    assert!(
        workflow.contains("infra_package_name=\"infra-images-${GIT_TAG}.tar.gz\""),
        "workflow 必须生成可解压合并到 release/images 的 infra 包"
    );
    assert!(
        deploy_script.contains("assert_required_images_available"),
        "deploy.sh 必须在 docker compose up 前检查所有运行镜像已离线可用"
    );
    assert!(
        deploy_script.contains("部署脚本不会从外部 registry 拉取镜像"),
        "缺少 infra 镜像时必须 fail-fast，不能让服务器尝试外网拉取"
    );
    assert!(
        deploy_script.contains("APP_ROLLBACK_IMAGE_REF"),
        "deploy.sh 应通过旧应用镜像标签回滚应用容器"
    );
    assert!(
        deploy_script.contains("up -d --no-recreate postgres redis"),
        "依赖容器必须原地启动，不能在日常 app 发布中重建 PostgreSQL / Redis"
    );
    assert!(
        !deploy_script.contains("docker rename"),
        "Compose 管理的容器不能通过 docker rename 备份，否则 Compose 会继续按 service label 识别旧容器"
    );
}

#[tokio::test]
async fn smoke_contract_tracks_required_headers() {
    let targets = load_smoke_targets();

    for expected_header in [
        "content-security-policy",
        "cross-origin-opener-policy",
        "cross-origin-resource-policy",
        "permissions-policy",
        "referrer-policy",
        "x-content-type-options",
        "x-frame-options",
    ] {
        assert!(
            targets
                .required_headers
                .iter()
                .any(|(name, _)| name == expected_header),
            "smoke 契约缺少必需响应头: {expected_header}"
        );
    }
}

fn assert_required_headers(headers: &HeaderMap, targets: &SmokeTargets, endpoint: &str) {
    for (header_name, expected_value) in &targets.required_headers {
        let actual = headers
            .get(header_name.as_str())
            .unwrap_or_else(|| panic!("端点 {endpoint} 缺少响应头 {header_name}"))
            .to_str()
            .unwrap_or_else(|error| {
                panic!("端点 {endpoint} 的响应头 {header_name} 不是合法字符串: {error}")
            });

        assert_eq!(
            actual, expected_value,
            "端点 {endpoint} 的响应头 {header_name} 应与 smoke 契约保持一致"
        );
    }
}
