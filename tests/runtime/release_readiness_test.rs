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
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let targets = load_smoke_targets();
    let smoke_script =
        fs::read_to_string(repo_root.join("deploy/smoke.sh")).expect("应能读取 smoke.sh");
    let security_headers = fs::read_to_string(repo_root.join("src/middleware/security_headers.rs"))
        .expect("应能读取安全响应头中间件");

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
    assert!(
        security_headers.contains("AppConfig::global().captcha.api_url")
            && security_headers.contains("captcha_origin_from_api_url"),
        "Content-Security-Policy 必须从 captcha.api_url 派生 connect-src origin，避免公开配置和 CSP 漂移"
    );
    assert!(
        smoke_script.contains("override_captcha_csp_header")
            && smoke_script.contains("APP__CAPTCHA__API_URL")
            && smoke_script.contains("SMOKE_REQUIRED_HEADER__CONTENT_SECURITY_POLICY"),
        "release smoke 必须在显式覆盖 captcha API URL 时同步调整 CSP 期望值"
    );
}

#[test]
fn release_packaging_splits_app_and_infra_artifacts() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workflow = fs::read_to_string(repo_root.join(".github/workflows/docker-build.yml"))
        .expect("应能读取 release workflow");
    let deploy_script =
        fs::read_to_string(repo_root.join("deploy/deploy.sh")).expect("应能读取 deploy.sh");
    let release_env = fs::read_to_string(repo_root.join("deploy/release.env.example"))
        .expect("应能读取 release env 示例");

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
        workflow.contains("\"app_archive_file\": \"${app_archive}.gz\"")
            && workflow.contains("\"app_archive_sha256\": \"${app_archive_sha256}\"")
            && workflow.contains("\"app_image_digest\": \"${app_image_digest}\""),
        "release manifest 必须记录 app 镜像归档文件名、归档 SHA256 和镜像摘要"
    );
    assert!(
        deploy_script.contains("assert_required_images_available"),
        "deploy.sh 必须在 docker compose up 前检查所有运行镜像已离线可用"
    );
    assert!(
        deploy_script.contains("verify_release_archives")
            && deploy_script.contains("sha256sum")
            && deploy_script.contains("assert_required_image_digests"),
        "deploy.sh 必须校验 release manifest 中的归档 SHA256 和加载后的镜像摘要"
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
        "依赖容器默认必须原地启动，不能在日常 app 发布中重建 PostgreSQL / Redis"
    );
    assert!(
        deploy_script.contains("DEPLOY_RECREATE_BASE_SERVICES")
            && deploy_script.contains("up -d --force-recreate postgres redis")
            && release_env.contains("DEPLOY_RECREATE_BASE_SERVICES=false"),
        "基础服务镜像升级必须通过显式门禁重建 PostgreSQL / Redis，避免误以为 no-recreate 已切换镜像"
    );
    assert!(
        !deploy_script.contains("docker rename"),
        "Compose 管理的容器不能通过 docker rename 备份，否则 Compose 会继续按 service label 识别旧容器"
    );
}

#[test]
fn local_docker_compose_uses_loopback_dependencies_without_self_connecting() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let local_compose = fs::read_to_string(repo_root.join("deploy/docker-compose.local.yml"))
        .expect("应能读取本地 Docker Compose");
    let build_script =
        fs::read_to_string(repo_root.join("deploy/build.sh")).expect("应能读取本地 Docker 脚本");

    assert!(
        local_compose.contains("postgres:16-alpine")
            && local_compose.contains("127.0.0.1:15432:5432")
            && local_compose.contains("APP__DATABASE__HOST=127.0.0.1")
            && local_compose.contains("APP__DATABASE__PORT=15432"),
        "本地 Docker 调试必须提供本地 PostgreSQL，并让 app 连接到宿主机回环端口而不是容器自身 127.0.0.1:5432"
    );
    assert!(
        local_compose.contains("127.0.0.1:16379:6379")
            && local_compose.contains("APP__REDIS__HOST=127.0.0.1")
            && local_compose.contains("APP__REDIS__PORT=16379"),
        "本地 Docker 调试必须让 Redis 同样走宿主机回环端口，保持 development 本地依赖约束"
    );
    assert!(
        local_compose.contains("network_mode: host")
            && local_compose.contains("APP__SERVER__PORT=11450")
            && local_compose.contains("http://127.0.0.1:11450/api/health"),
        "app 容器必须显式使用 host network 和 11450 健康检查，避免端口映射与 development 回环语义漂移"
    );
    assert!(
        build_script.contains("http://127.0.0.1:11450"),
        "本地 Docker 脚本输出的访问地址必须与 compose 中的 app 端口保持一致"
    );
}

#[test]
fn background_bulk_jobs_are_memory_bounded() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fetcher = fs::read_to_string(repo_root.join("src/infrastructure/electricity/fetcher.rs"))
        .expect("应能读取电费批量获取器");
    let fetcher_service =
        fs::read_to_string(repo_root.join("src/domain/services/electricity_fetcher_service.rs"))
            .expect("应能读取电费获取服务");
    let history_repository = fs::read_to_string(
        repo_root.join("src/infrastructure/repositories/electricity_history_repository.rs"),
    )
    .expect("应能读取电费历史记录仓储");
    let room_repository =
        fs::read_to_string(repo_root.join("src/infrastructure/repositories/room_repository.rs"))
            .expect("应能读取房间仓储");
    let room_path_tree =
        fs::read_to_string(repo_root.join("src/domain/services/room_path_tree.rs"))
            .expect("应能读取房间路径树");
    let bootstrap_app =
        fs::read_to_string(repo_root.join("src/bootstrap/app.rs")).expect("应能读取应用启动入口");
    let bootstrap_runtime =
        fs::read_to_string(repo_root.join("src/bootstrap/runtime.rs")).expect("应能读取启动运行时");
    let room_use_case = fs::read_to_string(repo_root.join("src/modules/room/application/mod.rs"))
        .expect("应能读取房间访问用例");
    let redis_writer =
        fs::read_to_string(repo_root.join("src/infrastructure/redis/batch_writer.rs"))
            .expect("应能读取 Redis 批量写入器");
    let notification_service =
        fs::read_to_string(repo_root.join("src/domain/services/notification_service.rs"))
            .expect("应能读取通知服务");
    let release_compose = fs::read_to_string(repo_root.join("deploy/compose.release.yml"))
        .expect("应能读取 release compose");

    assert!(
        fetcher.contains(".buffer_unordered(self.max_concurrent)"),
        "全量电费抓取必须使用流式背压限制在固定并发内"
    );
    assert!(
        !fetcher.contains("tokio::spawn(async move { fetcher.fetch_one(room_id).await })"),
        "全量电费抓取不能提前为所有房间创建 Tokio task"
    );
    assert!(
        fetcher_service.contains("fetch_task_lock"),
        "定时和手动电费抓取必须共享互斥保护，避免多轮全量任务叠加"
    );
    assert!(
        fetcher_service.contains("try_run_fetch_task"),
        "定时电费抓取必须在上一轮未结束时跳过本轮，而不是排队堆积"
    );
    assert!(
        !redis_writer.contains("collect::<Vec<_>>()"),
        "Redis 批量写入不能为全量电费结果再构造临时 Vec 索引"
    );
    assert!(
        history_repository.contains("INSERT INTO electricity_history"),
        "电费历史快照应直接让数据库执行 INSERT ... SELECT，避免把所有房间先载入 Rust 堆"
    );
    assert!(
        history_repository.contains("SELECT roomid, electricity_fee"),
        "电费历史快照应直接从 rooms 表投影所需列，而不是构造中间 Vec<NewElectricityHistory>"
    );
    assert!(
        !history_repository.contains("NewElectricityHistory"),
        "电费历史仓储不应再依赖 Rust 侧逐条构造历史记录"
    );
    assert!(
        bootstrap_app.contains("initialize_path_tree(&state, &db_pool).await;"),
        "应用启动仍应初始化路径树"
    );
    assert!(
        !bootstrap_app.contains("warm_cache(roomids)"),
        "应用启动不应再预热全量 Room/Binding 缓存"
    );
    assert!(
        bootstrap_runtime.contains("find_all_active_path_entries"),
        "路径树初始化应只加载最小活跃路径字段"
    );
    assert!(
        room_repository.contains("find_all_active_path_entries"),
        "房间仓储应提供路径树最小字段查询"
    );
    assert!(
        room_path_tree.contains("build_from_primary_paths"),
        "路径树应支持直接从轻量 path entries 构建"
    );
    assert!(
        !bootstrap_runtime.contains("flagged_rooms_cache_refresher"),
        "启动后台任务不应再维护全量 flagged rooms 缓存"
    );
    assert!(
        !room_use_case.contains("flagged_rooms_cache"),
        "房间访问用例不应再依赖后台 flagged rooms 缓存"
    );
    assert!(
        notification_service.contains("MissedTickBehavior::Delay"),
        "周期通知任务耗时超过间隔时应延后下一轮，避免补偿式突发执行"
    );
    assert!(
        !notification_service.contains("Arc::new(user_map.clone())"),
        "通知发送不能为每个房间克隆整份用户映射"
    );
    assert!(
        release_compose.contains("MIMALLOC_PURGE_DELAY"),
        "release 容器必须显式启用 mimalloc 及时归还空闲物理页的配置入口"
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
