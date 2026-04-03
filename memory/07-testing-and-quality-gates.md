# Electricity Monitor 仓库记忆：测试真源与质量门禁

## 当前测试真源
- 后端源码内测试入口：`cargo test --lib`
- 后端认证契约入口：`tests/contracts/auth_integration_test.rs`
- runtime / readiness 契约入口：`tests/runtime/release_readiness_test.rs`
- 共享测试支撑：`tests/support/`
- 环境型测试分层目录：`tests/infra/`
- 测试文档真源：`docs/guides/TESTING.md`
- PR / 手动质量门禁工作流：`.github/workflows/ci.yml`

## 当前本地可执行的附加审计门禁
- 前端依赖审计：`pnpm --dir frontend audit --prod`
- Rust 依赖审计：`cargo audit`
- 最近一次基线结果：
  - `pnpm audit --prod` 命中 `7` 条 advisory（`5` 高危、`2` 中危），重点涉及 `axios`、`react-router`、`rollup`、`picomatch`
  - `cargo audit` 命中 `5` 条漏洞与 `2` 条警告，重点涉及 `bytes`、`rkyv`、`rsa`、`rustls-webpki`、`time`、`bincode`、`lru`
- 当前这些审计路径已在本地验证可执行，但还没有接入 `.github/workflows/ci.yml`

## 后端测试约束
- 认证集成测试必须走真实 `/api/auth/verify-and-login` 链路；不要回退到本地签发 JWT 伪造登录成功。
- `tests/support/app_factory.rs` 负责统一装配 `AppState` 与 Router，避免顶层集成测试重复拼装 DB / Redis / Cache / RateLimiter。
- `tests/support/auth_fixture.rs` 通过写入 Redis 验证码来驱动真实登录链，再访问受保护接口。
- `tests/support/seed.rs` 负责测试房间 seed，避免 auth/binding 契约测试重复造数。
- 当前 auth/binding 契约已覆盖：
  - `/api/auth/send-verification-code` 的无效 captcha token 拒绝逻辑
  - 登录成功 / 登录失败
  - Bearer 前缀校验
  - refresh token
  - `/api/bindings` 管理员空列表
  - 普通用户绑定 CRUD
  - 绑定越权访问
  - admin 禁止创建绑定
- `tests/contracts/send_verification_code_integration_test.rs` 通过本地 mock NapCat HTTP API 覆盖：
  - 发送验证码成功后 Redis 写入验证码
  - `USER_NOT_FRIEND` 错误映射

## readiness / smoke 契约
- `deploy/smoke.targets` 是本地 readiness test 与 release smoke 共用的检查目标真源。
- 当前共享检查目标包含：
  - `/api/health`
  - `/api/health/db`
  - `/`
  - `release-manifest.json`
  - `deploy-result.json`
- 如果修改健康检查路径、静态入口或 release 产物文件名，必须先更新 `deploy/smoke.targets`，再同步测试、脚本与文档。

## 外部依赖测试策略
- 房间树和电费抓取保留真实测试，并要求在需要时通过 `RUN_EXTERNAL_INTEGRATION_TESTS=1` 显式执行。
- 在真实测试之外，当前已补齐 mock 回归：
  - `RoomClient` mock 成功与重试
  - `RoomBatchFetcher` mock HTTP + parser 集成
  - `CaptchaVerificationService` mock 第三方验证码 API
  - `/api/auth/send-verification-code` mock QQ API
- 真实 NapCat HTTP 服务联通性验证应在私有运维环境完成，验证记录不写回公开仓库的 memory 或文档。
- 原则是：真实测试验证真实外部契约，mock 测试承担稳定回归。

## CI 门禁现状
- `backend-tests`：
  - PostgreSQL / Redis service containers
  - 先复制 `config/development.toml.example` 为 `config/default.toml`
  - 再把 `config/default.toml` 中的开发密码占位值替换为 CI 内的 `postgres`
  - `RUN_INTEGRATION_TESTS=1`
  - `REDIS_HOST=127.0.0.1`
  - `REDIS_PORT=6379`
  - `cargo run --bin migrate`
  - `cargo test --lib`
  - `cargo test --test auth_integration_test`
  - `cargo test --test send_verification_code_integration_test`
  - `cargo test --test release_readiness_test`
- `backend-external-tests`：
  - 仅在手动 workflow 且显式开启时执行
  - 先复制 `config/development.toml.example` 为 `config/default.toml`
  - `RUN_EXTERNAL_INTEGRATION_TESTS=1`
  - 真实房间树测试
  - 真实电费抓取测试
- `frontend-quality`：
  - `pnpm install --frozen-lockfile`
  - `pnpm lint`
  - `pnpm build:prod`
- `frontend-tests`：
  - `pnpm install --frozen-lockfile`
  - `pnpm test`
- `architecture-guard`：
  - `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`
- 所有 job 都会上传对应日志 artifact，便于失败定位。
- 当前 CI 仍未接入：
  - `cargo audit`
  - `pnpm --dir frontend audit --prod`

## 最近一次本地质量基线
- `cargo test --lib`：`116 passed`
- `cargo test --test auth_integration_test`：`10 passed`
- `cargo test --test send_verification_code_integration_test`：`1 passed`
- `cargo test --test release_readiness_test`：`2 passed`
- `pnpm --dir frontend lint`：通过
- `pnpm --dir frontend test`：`3` 个文件、`6` 个用例通过
- `pnpm --dir frontend build:prod`：通过，但仍有大 chunk warning
- `cargo clippy --all-targets -- -D warnings`：失败，当前稳定阻塞点在 `src/domain/services/notification_gate.rs`
- `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`：当前会把 `frontend/src/**/AGENTS.md` 文本误报成 `@/services/api` 违规导入

## 本地执行约定补充
- 本地运行前先从 `config/development.toml.example` 复制生成 `config/default.toml`；不要再直接编辑仓库模板以保存local environment参数。
- 若local environment PostgreSQL 密码与开发模板不一致，直接修改本地 `config/default.toml` 中的 `database.password`。
- `scripts/backend-checks.ps1` 会检查 `config/default.toml` 是否仍保留模板占位值，并统一执行迁移与后端关键回归。
- 该约定适用于：
  - `cargo test --test auth_integration_test`
  - `cargo test --test send_verification_code_integration_test`
  - `cargo test --test release_readiness_test`

## 当前仍未完成的测试项
- 源码内依赖环境变量启用的 infra 测试还未全部迁移到 `tests/infra/` 独立 target。
- 外部网络测试已拆到 `RUN_EXTERNAL_INTEGRATION_TESTS=1`，并已形成手动 workflow lane；但还未独立成单独 Rust test target。
- `cargo-nextest` 仍处于评估后暂不接入状态。
- Playwright 浏览器 smoke 仍处于评估后暂不接入状态。
- 真实 Linux Docker 主机上的 smoke / 回滚演练仍需人工执行，不能被本地 readiness test 代替。
