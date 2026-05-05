# 测试指南

## 当前测试真相

当前仓库的默认质量门禁由以下入口组成：

- Rust 单元/源码内测试：`cargo test --lib`
- 认证契约测试：`cargo test --test auth_integration_test`
- runtime / readiness 契约测试：`cargo test --test release_readiness_test`
- 前端行为测试：在 `frontend/` 目录执行 `bun run test`
- 前端质量检查：在 `frontend/` 目录执行 `bun run lint`、`bun run build:prod`、`bun run check:bundle`
- 架构守护：`powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`
- Linux 后端统一自检：`bash scripts/backend-checks.sh`
- Windows 后端统一自检：`powershell -ExecutionPolicy Bypass -File scripts/backend-checks.ps1`
- Pull Request / 手动门禁：`.github/workflows/ci.yml`

这份文档只描述当前仓库里已经存在且可执行的测试入口。

## 当前分层

### 后端

- `src/**`：单元测试与少量带环境门槛的基础设施测试
- `tests/contracts/auth_integration_test.rs`：真实 `/api/auth/verify-and-login` 登录链、`/api/auth/me`、`/api/auth/refresh`、`/api/bindings` CRUD、越权访问与 admin 限制契约，同时覆盖 refresh cookie 轮换、refresh token 误用为 Bearer、access token 误用为 refresh、logout 清理 cookie
- `tests/runtime/release_readiness_test.rs`：读取 `deploy/smoke.targets`，校验 health / db health / 静态入口、必需文件与统一响应安全头契约
- `tests/contracts/send_verification_code_integration_test.rs`：通过本地 mock NapCat HTTP API 覆盖 `/api/auth/send-verification-code` 的成功发送与 `USER_NOT_FRIEND` 分支
- `tests/support/`：共享 app factory、登录 fixture、seed helper、smoke 契约读取
- `tests/infra/`：环境型独立 test target 的预留分层；当前目录中记录了仍在源码内的 infra 覆盖位置

### 前端

- 已接入 `Vitest + React Testing Library + MSW`
- 首批自动回归覆盖：
  - `src/App.test.tsx`
  - `src/entities/binding/api/bindingApi.test.ts`
  - `src/pages/LoginPage.test.tsx`
  - `src/features/bind-room/model/useBindRoomModal.test.tsx`
  - `src/features/dashboard/model/useDashboardPage.test.tsx`

### 发布 smoke

- `deploy/smoke.targets`：release smoke 与 readiness test 的检查目标真源
- `deploy/smoke.sh`：目标环境 smoke，读取同一份 `smoke.targets`

## 本地前置条件

运行后端契约测试前，需要满足：

1. 本地 PostgreSQL 与 Redis 已启动；它们可以是系统服务，也可以是映射到 `127.0.0.1:5432` / `127.0.0.1:6379` 的 Docker 容器
2. 已从 `config/development.toml.example` 复制生成 `config/development.toml`
3. 已将 `config/development.toml` 中的 `database.password` 改成当前本地 PostgreSQL 的真实密码或非空开发值
4. `APP_ENV=development`
5. 已执行迁移：`cargo run --bin migrate`

开发环境只允许连接本地 PostgreSQL / Redis；不要把 `development` 指向远端库。

## 推荐命令矩阵

### 日常快速回归

Linux：

```bash
cargo test --lib
cd frontend
bun run test
bun run lint
bun run check:bundle
bun audit
cd ..
```

Windows 原生：

```powershell
cargo test --lib
cd frontend
bun run test
bun run lint
bun run check:bundle
bun audit
cd ..
powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1
```

### 后端关键链路回归

Linux：

```bash
cp config/development.toml.example config/development.toml
# 继续编辑 config/development.toml，把 database.password 改成当前本地 PostgreSQL 的真实密码或非空开发值
export APP_ENV=development
cargo run --bin migrate
cargo clippy --all-targets -- -D warnings
cargo test --test auth_integration_test
cargo test --test send_verification_code_integration_test
cargo test --test release_readiness_test
```

Windows 原生：

```powershell
Copy-Item config/development.toml.example config/development.toml
# 继续编辑 config/development.toml，把 database.password 改成当前本地 PostgreSQL 的真实密码或非空开发值
$env:APP_ENV="development"
cargo run --bin migrate
cargo clippy --all-targets -- -D warnings
cargo test --test auth_integration_test
cargo test --test send_verification_code_integration_test
cargo test --test release_readiness_test
```

如果希望使用统一入口，也可以运行：

```bash
bash scripts/backend-checks.sh
```

```powershell
powershell -ExecutionPolicy Bypass -File scripts/backend-checks.ps1
```

### 前端构建回归

```bash
cd frontend
bun run test
bun run build:prod
bun run check:bundle
```

`build:prod` 会先生成 `frontend/dist/`，再复制到仓库根目录 `static/`，供后端静态文件服务与 Docker 构建使用。`check:bundle` 会对 `dist/assets/*.js` 执行细粒度体积预算检查，不再只依赖 Vite 的默认 chunk warning。

## 环境型测试说明

仓库里仍有一批源码内环境型测试通过 `RUN_INTEGRATION_TESTS=1` 或 Redis 连接变量显式启用，例如：

- `src/infrastructure/database/pool.rs`
- `src/infrastructure/redis/pool.rs`
- `src/domain/services/rate_limiter.rs`
- `src/infrastructure/repositories/room_repository.rs`
- `src/domain/services/room_sync/crawler/client.rs`

这些测试目前仍属于“可选 infra 覆盖”，不是默认 PR 门禁。启用示例：

Linux：

```bash
export RUN_INTEGRATION_TESTS=1
cargo test --lib
```

如果同时需要 Redis 单独覆盖，也可以显式设置：

```bash
export REDIS_HOST=127.0.0.1
export REDIS_PORT=6379
cargo test --lib
```

Windows 原生：

```powershell
$env:RUN_INTEGRATION_TESTS="1"
cargo test --lib
```

```powershell
$env:REDIS_HOST="127.0.0.1"
$env:REDIS_PORT="6379"
cargo test --lib
```

外部网络测试已从默认 infra 门禁中分离，避免 CI 因公网依赖失真。当前需要显式启用的外部测试包括：

- `src/domain/services/room_sync/crawler/client.rs::test_fetch_room_tree`
- `src/infrastructure/electricity/parser.rs::test_parse_real_api_response`

这些链路当前采用“真实测试 + mock 测试”双轨：

- 房间树：
  - 真实：`test_fetch_room_tree`
  - mock：`test_fetch_room_tree_with_mock_server`、`test_fetch_tree_retries_until_success_with_mock_server`
- 电费抓取：
  - 真实：`test_parse_real_api_response`
  - mock：`test_fetch_batch_with_mock_server_filters_failures`

启用方式：

Linux：

```bash
export RUN_EXTERNAL_INTEGRATION_TESTS=1
cargo test --lib
```

Windows 原生：

```powershell
$env:RUN_EXTERNAL_INTEGRATION_TESTS="1"
cargo test --lib
```

云服务依赖已改为 mock 驱动的自动回归：

- NapCat HTTP 机器人服务发送链：`cargo test --test send_verification_code_integration_test`
- 第三方验证码校验服务：`test_verify_captcha_with_mock_server_*`

## CI 门禁

`.github/workflows/ci.yml` 当前包含三类 job：

- `backend-tests`
  - 启动 PostgreSQL / Redis service containers
  - 复制 `config/development.toml.example` 为 `config/development.toml`
  - 将 `config/development.toml` 中的开发密码占位值替换为 CI 内的 `postgres`
  - 执行 `cargo run --bin migrate`
  - 运行 `cargo test --lib`
  - 运行 `cargo test --test auth_integration_test`
  - 运行 `cargo test --test send_verification_code_integration_test`
  - 运行 `cargo test --test release_readiness_test`
  - 运行 `cargo clippy --all-targets -- -D warnings`
  - 默认设置 `RUN_INTEGRATION_TESTS=1` 与 Redis 连接变量，确保本地 DB/Redis 相关测试不再被短路
- `backend-external-tests`
  - 仅在 `workflow_dispatch` 且 `run_external_integration_tests=true` 时执行
  - 运行真实房间树测试
  - 运行真实电费抓取测试
- `frontend-quality`
  - 执行 `bun install --frozen-lockfile`
  - 运行 `bun run lint`
  - 运行 `bun audit`
  - 运行 `bun run build:prod`
  - 运行 `bun run check:bundle`
- `frontend-tests`
  - 执行 `bun install --frozen-lockfile`
  - 运行 `bun run test`
- `dependency-audit`
  - 安装并运行 `cargo audit -q`
  - 上传审计 artifact 并将摘要写入 workflow summary
  - 当前为 Rust 依赖阻断项，失败会直接阻断 workflow
- `architecture-guard`
  - 运行 `scripts/check-architecture.ps1`

各 job 当前都会上传对应日志 artifact，便于失败定位。

## readiness 与 smoke 的关系

- `tests/runtime/release_readiness_test.rs` 在本地 / CI 中读取 `deploy/smoke.targets`
- `deploy/smoke.sh` 在 release 目录中读取同一份 `smoke.targets`
- 当前共用的检查目标包括：
  - `/api/health`
  - `/api/health/db`
  - `/`
  - `release-manifest.json`
  - `deploy-result.json`
  - 一组统一响应安全头

这意味着如果你修改健康检查路径、release 产物文件名或统一响应安全头，必须同时更新 `deploy/smoke.targets`，而不是只改单边硬编码。

## 当前缺口

以下事项仍然未在默认阻断门禁中完成：

- 把源码内的环境型测试逐步迁移成 `tests/infra/` 下的独立 test target
- `cargo-nextest` 试点
- Playwright 浏览器 smoke
- 外部网络测试的独立 workflow / 独立 test target
- 真实 Linux Docker 主机上的部署与回滚演练

这些都应作为下一批次继续推进，但不要在当前仓库里假装已经存在。
