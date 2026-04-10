# Electricity Monitor 仓库记忆：测试真源与质量门禁

## 测试真源

- 后端源码内测试入口：`cargo test --lib`
- 后端认证契约入口：`tests/contracts/auth_integration_test.rs`
- 验证码发送契约入口：`tests/contracts/send_verification_code_integration_test.rs`
- runtime / readiness 契约入口：`tests/runtime/release_readiness_test.rs`
- 共享测试支撑：`tests/support/`
- 环境型测试分层目录：`tests/infra/`
- 测试文档真源：`docs/guides/TESTING.md`
- PR / 手动质量门禁工作流：`.github/workflows/ci.yml`

## 前端与依赖审计入口

- 在 `frontend/` 目录执行 `bun run test`、`bun run lint`、`bun run build:prod`。
- 在 `frontend/` 目录执行 `bun audit` 做前端依赖审计。
- 在仓库根目录执行 `cargo audit` 做 Rust 依赖审计。
- 在仓库根目录执行 `cargo clippy --all-targets -- -D warnings` 做后端代码质量阻断检查。

## 后端测试约束

- 认证集成测试必须走真实 `/api/auth/verify-and-login` 链路；不要回退到本地签发 JWT 伪造登录成功。
- `tests/support/app_factory.rs` 负责统一装配 `AppState` 与 Router，避免顶层集成测试重复拼装依赖。
- `tests/support/auth_fixture.rs` 通过写入 Redis 验证码来驱动真实登录链，再访问受保护接口。
- `tests/support/seed.rs` 负责测试房间 seed，避免 auth / binding 契约测试重复造数。
- `/api/auth/send-verification-code` 的契约测试通过本地 mock NapCat HTTP API 覆盖成功发送与错误映射。

## readiness / smoke 契约

- `deploy/smoke.targets` 是本地 readiness test 与 release smoke 共用的检查目标真源。
- 共享检查目标包含 `/api/health`、`/api/health/db`、`/`、`release-manifest.json` 与 `deploy-result.json`。
- 如果修改健康检查路径、静态入口或 release 产物文件名，必须先更新 `deploy/smoke.targets`，再同步测试、脚本与文档。

## 外部依赖测试策略

- 房间树和电费抓取保留真实测试，并要求在需要时通过 `RUN_EXTERNAL_INTEGRATION_TESTS=1` 显式执行。
- 在真实测试之外，mock 回归承担稳定回归职责，覆盖房间树、批量抓取、验证码校验和 QQ 发送链路。
- 真实 NapCat HTTP 服务联通性验证应在私有运维环境完成，验证记录不写回公开仓库的 memory 或文档。

## CI 门禁结构

- `backend-tests`：准备本地 PostgreSQL / Redis、复制开发模板、执行迁移、后端关键回归与 `cargo clippy --all-targets -- -D warnings`。
- `backend-external-tests`：仅在手动 workflow 且显式开启时执行真实外部测试。
- `frontend-quality`：在 `frontend/` 中执行 `bun install --frozen-lockfile`、`bun run lint`、`bun run build:prod`。
- `frontend-tests`：在 `frontend/` 中执行 `bun install --frozen-lockfile`、`bun run test`。
- `dependency-audit`：执行 `cargo audit -q` 与 `bun audit --json`，上传审计 artifact 并写入 workflow summary；当前作为报告项，不阻断 workflow。
- `architecture-guard`：执行 `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`。
- 所有 job 都会上传对应日志 artifact，便于失败定位。

## 本地执行约定

- 本地运行前先从 `config/development.toml.example` 复制生成 `config/development.toml`，不要直接编辑仓库模板保存local environment参数。
- 若local environment PostgreSQL 密码与开发模板不一致，直接修改本地 `config/development.toml` 中的 `database.password`。
- `scripts/backend-checks.ps1` 会检查 `config/development.toml` 是否仍保留模板占位值，并统一执行迁移与后端关键回归。

## 仍未接入默认阻断门禁的验证

- `cargo audit`
- `bun audit`
- 尚未全部迁移完成的 `tests/infra/` 独立 target
- 真实 Linux Docker 主机上的 smoke / 回滚演练
