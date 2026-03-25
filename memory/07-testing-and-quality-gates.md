# Electricity Monitor 仓库记忆：测试真源与质量门禁

## 当前测试真源
- 后端源码内测试入口：`cargo test --lib`
- 后端认证契约入口：`tests/auth_integration_test.rs`
- runtime / readiness 契约入口：`tests/release_readiness_test.rs`
- 共享测试支撑：`tests/support/`
- 测试文档真源：`docs/guides/TESTING.md`
- PR / 手动质量门禁工作流：`.github/workflows/ci.yml`

## 后端测试约束
- 认证集成测试必须走真实 `/api/auth/verify-and-login` 链路；不要回退到本地签发 JWT 伪造登录成功。
- `tests/support/app_factory.rs` 负责统一装配 `AppState` 与 Router，避免顶层集成测试重复拼装 DB / Redis / Cache / RateLimiter。
- `tests/support/auth_fixture.rs` 通过写入 Redis 验证码来驱动真实登录链，再访问受保护接口。
- `/api/bindings` 当前对管理员返回稳定空数组，对新普通用户在未绑定房间时也返回空数组；这两条行为现在有自动测试保护。

## readiness / smoke 契约
- `deploy/smoke.targets` 是本地 readiness test 与 release smoke 共用的检查目标真源。
- 当前共享检查目标包含：
  - `/api/health`
  - `/api/health/db`
  - `/`
  - `release-manifest.json`
  - `deploy-result.json`
- 如果修改健康检查路径、静态入口或 release 产物文件名，必须先更新 `deploy/smoke.targets`，再同步测试、脚本与文档。

## CI 门禁现状
- `backend-tests`：
  - PostgreSQL / Redis service containers
  - `cargo run --bin migrate`
  - `cargo test --lib`
  - `cargo test --test auth_integration_test`
  - `cargo test --test release_readiness_test`
- `frontend-quality`：
  - `pnpm install --frozen-lockfile`
  - `pnpm lint`
  - `pnpm build:prod`
- `architecture-guard`：
  - `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`

## 当前仍未完成的测试项
- 前端行为测试 runner / setup / mock 基础设施尚未接入。
- `.github/workflows/ci.yml` 目前还没有前端行为测试 job。
- `tests/` 目录还未进一步拆分到 `contracts/`、`runtime/`、`infra/`。
- 真实 Linux Docker 主机上的 smoke / 回滚演练仍需人工执行，不能被本地 readiness test 代替。
