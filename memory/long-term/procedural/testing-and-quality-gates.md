---
type: procedural
status: verified
scope: 测试与质量门禁
updated_at: 2026-05-05
verified_at: 2026-05-05
sources:
  - docs/guides/TESTING.md
  - .github/workflows/ci.yml
  - deploy/smoke.targets
  - scripts/backend-checks.ps1
  - scripts/backend-checks.sh
summary: 后端测试真源、前端检查入口、CI 门禁结构和本地执行约定
---

# Electricity Monitor 测试真源与质量门禁

## 适用场景

- 需要运行后端回归、前端回归、依赖审计、clippy 门禁或 readiness / smoke 契约验证时。
- 需要调整 CI 工作流、测试入口、shared support 或本地检查脚本时。

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

- 在 `frontend/` 目录执行 `bun run test`、`bun run lint`、`bun run build:prod`、`bun run check:bundle`。
- 在 `frontend/` 目录执行 `bun audit` 做前端依赖审计。
- 在仓库根目录执行 `cargo audit -q` 做 Rust 依赖审计。
- 在仓库根目录执行 `cargo clippy --all-targets -- -D warnings` 做后端代码质量阻断检查。

## 后端测试约束

- 认证集成测试必须走真实 `/api/auth/verify-and-login` 链路；不要回退到本地签发 JWT 伪造登录成功。
- 认证集成测试覆盖 `/api/auth/send-verification-code` 必须携带一次性 captcha token；缺失 token 应在调用 QQ 机器人前被拒绝。
- 认证集成测试覆盖未绑定用户不能通过 `/api/rooms/by-path` 或 `/api/rooms/by-hash` 读取房间电费详情；路径树叶子节点只能提供绑定所需的最小 `roomid`。
- `tests/support/app_factory.rs` 负责统一装配 `AppState` 与 Router，避免顶层集成测试重复拼装依赖。
- `tests/support/auth_fixture.rs` 通过写入 Redis 验证码来驱动真实登录链，再访问受保护接口。
- `tests/support/seed.rs` 负责测试房间 seed，避免 auth / binding 契约测试重复造数。
- `/api/auth/send-verification-code` 的契约测试通过本地 mock NapCat HTTP API 覆盖成功发送与错误映射；测试直接种入一次性 captcha token，不依赖第三方验证码服务。

## readiness / smoke 契约

- `deploy/smoke.targets` 是本地 readiness test 与 release smoke 共用的检查目标真源。
- 共享检查目标包含 `/api/health`、`/api/health/db`、`/`、`release-manifest.json`、`deploy-result.json` 与一组统一响应安全头。
- `tests/runtime/release_readiness_test.rs` 额外覆盖 `/api/public-config`，确保公开运行时配置响应带统一安全头且不暴露完整 `qq_bot` 配置或 bearer token。
- 如果修改健康检查路径、静态入口或 release 产物文件名，必须先更新 `deploy/smoke.targets`，再同步测试、脚本与文档。

## CI 门禁结构

- `backend-tests`：准备本地 PostgreSQL / Redis、复制开发模板、执行迁移、后端关键回归与 `cargo clippy --all-targets -- -D warnings`。
- `backend-external-tests`：仅在手动 workflow 且显式开启时执行真实外部测试。
- `frontend-quality`：在 `frontend/` 中执行 `bun install --frozen-lockfile`、`bun run lint`、`bun audit`、`bun run build:prod`、`bun run check:bundle`。
- `frontend-tests`：在 `frontend/` 中执行 `bun install --frozen-lockfile`、`bun run test`。
- `dependency-audit`：执行 `cargo audit -q`，上传审计 artifact 并写入 workflow summary；当前为阻断项。
- `architecture-guard`：执行 `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`。
- 所有 job 都会上传对应日志 artifact，便于失败定位。

## 本地执行约定

- 本地运行前先从 `config/development.toml.example` 复制生成 `config/development.toml`，不要直接编辑仓库模板保存local environment参数。
- 若本地 PostgreSQL 密码与开发模板不一致，直接修改本地 `config/development.toml` 中的 `database.password`。
- `config/development.toml` 还必须填写 `qq_bot.public_qq_number`；它会被 `/api/public-config` readiness 覆盖校验并用于前端好友添加引导。
- `scripts/backend-checks.sh` 与 `scripts/backend-checks.ps1` 会检查 `config/development.toml` 是否仍保留模板占位值，并统一执行迁移与后端关键回归；前者用于 Linux，后者用于 Windows 原生环境。

## 仍未接入默认阻断门禁的验证

- 尚未全部迁移完成的 `tests/infra/` 独立 target。
- 真实 Linux Docker 主机上的 smoke / 回滚演练。

## 常见风险

- 真实外部依赖测试和运维环境 smoke 不能被本地 mock 或 readiness test 替代。
- 调整健康检查、静态入口或 release 产物命名时，如果漏改 `deploy/smoke.targets`，会同时破坏本地 readiness 与 release smoke。
