# 测试指南

## 当前测试真相

当前仓库的默认质量门禁由以下入口组成：

- Rust 单元/源码内测试：`cargo test --lib`
- 认证契约测试：`cargo test --test auth_integration_test`
- runtime / readiness 契约测试：`cargo test --test release_readiness_test`
- 前端质量检查：`pnpm --dir frontend lint`、`pnpm --dir frontend build:prod`
- 架构守护：`powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`
- Pull Request / 手动门禁：`.github/workflows/ci.yml`

这份文档只描述当前仓库里已经存在且可执行的测试入口；前端行为测试框架仍未落地，不在本文档里伪装成“已完成”。

## 当前分层

### 后端

- `src/**`：单元测试与少量带环境门槛的基础设施测试
- `tests/auth_integration_test.rs`：真实 `/api/auth/verify-and-login` 登录链、`/api/auth/me` 和 `/api/bindings` 权限契约
- `tests/release_readiness_test.rs`：读取 `deploy/smoke.targets`，校验 health / db health / 静态入口契约
- `tests/support/`：共享 app factory、登录 fixture、smoke 契约读取

### 前端

- 当前只有 `lint` 与 `build:prod`
- 尚未接入 Vitest / React Testing Library / MSW

### 发布 smoke

- `deploy/smoke.targets`：release smoke 与 readiness test 的检查目标真源
- `deploy/smoke.sh`：目标环境 smoke，读取同一份 `smoke.targets`

## 本地前置条件

运行后端契约测试前，需要满足：

1. 本地 PostgreSQL 与 Redis 已启动
2. `APP_ENV=development`
3. 已执行迁移：`cargo run --bin migrate`

开发环境只允许连接本地 PostgreSQL / Redis；不要把 `development` 指向远端库。

## 推荐命令矩阵

### 日常快速回归

```powershell
cargo test --lib
pnpm --dir frontend lint
powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1
```

### 后端关键链路回归

```powershell
$env:APP_ENV="development"
cargo run --bin migrate
cargo test --test auth_integration_test
cargo test --test release_readiness_test
```

### 前端构建回归

```powershell
pnpm --dir frontend build:prod
```

`build:prod` 会先生成 `frontend/dist/`，再复制到仓库根目录 `static/`，供后端静态文件服务与 Docker 构建使用。

## 环境型测试说明

仓库里仍有一批源码内环境型测试通过 `RUN_INTEGRATION_TESTS=1` 或 Redis 连接变量显式启用，例如：

- `src/infrastructure/database/pool.rs`
- `src/infrastructure/redis/pool.rs`
- `src/domain/services/rate_limiter.rs`
- `src/infrastructure/repositories/room_repository.rs`
- `src/domain/services/room_sync/crawler/client.rs`

这些测试目前仍属于“可选 infra 覆盖”，不是默认 PR 门禁。启用示例：

```powershell
$env:RUN_INTEGRATION_TESTS="1"
cargo test --lib
```

如果同时需要 Redis 单独覆盖，也可以显式设置：

```powershell
$env:REDIS_HOST="127.0.0.1"
$env:REDIS_PORT="6379"
cargo test --lib
```

## CI 门禁

`.github/workflows/ci.yml` 当前包含三类 job：

- `backend-tests`
  - 启动 PostgreSQL / Redis service containers
  - 执行 `cargo run --bin migrate`
  - 运行 `cargo test --lib`
  - 运行 `cargo test --test auth_integration_test`
  - 运行 `cargo test --test release_readiness_test`
- `frontend-quality`
  - 执行 `pnpm install --frozen-lockfile`
  - 运行 `pnpm lint`
  - 运行 `pnpm build:prod`
- `architecture-guard`
  - 运行 `scripts/check-architecture.ps1`

当前 CI 还没有前端行为测试 job，因为对应测试基础设施尚未接入。

## readiness 与 smoke 的关系

- `tests/release_readiness_test.rs` 在本地 / CI 中读取 `deploy/smoke.targets`
- `deploy/smoke.sh` 在 release 目录中读取同一份 `smoke.targets`
- 当前共用的检查目标包括：
  - `/api/health`
  - `/api/health/db`
  - `/`
  - `release-manifest.json`
  - `deploy-result.json`

这意味着如果你修改健康检查路径或 release 产物文件名，必须同时更新 `deploy/smoke.targets`，而不是只改单边硬编码。

## 当前缺口

以下事项仍然未在本批次内完成：

- 前端行为测试 runner / setup / mock 基础设施
- 独立的前端行为测试 job
- `tests/` 目录进一步细分到 `contracts/`、`runtime/`、`infra/`
- 真实 Linux Docker 主机上的部署与回滚演练

这些都应作为下一批次继续推进，但不要在当前仓库里假装已经存在。
