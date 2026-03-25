mod support;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use support::{
    app_factory::create_test_app,
    smoke_contract::{load_smoke_targets, smoke_targets_path},
};

#[tokio::test]
async fn runtime_endpoints_from_smoke_contract_pass() {
    let test_app = create_test_app().await;
    let targets = load_smoke_targets();

    for endpoint in [
        targets.health_endpoint,
        targets.db_health_endpoint,
        targets.static_entry,
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
    }
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
}
