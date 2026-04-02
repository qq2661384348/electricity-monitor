# 项目协作 AGENTS

## 作用范围

- 根目录 `AGENTS.md` 只定义全仓共享规则；进入 `src/`、`src/modules/`、`frontend/src/`、`frontend/src/features/`、`frontend/src/entities/` 后，必须同时遵守就近的子级 `AGENTS.md`。
- 每次任务开始前先阅读 `./memory/`；交付前同步更新受影响的 `memory/*.md`，避免仓库状态和项目记忆漂移。
- 先以代码、配置、CI、脚本和当前文档真源为准；不要把历史说明或已失效流程重新写回仓库。

## 仓库地图

- `src/`: Rust + Axum 后端主代码；启动装配在 `src/bootstrap/`，共享运行时资源在 `src/state.rs`。
- `src/modules/`: 后端模块化迁移主线；新的复杂编排优先进入 `src/modules/*/application`。
- `frontend/`: React 19 + Vite 7 前端；生产构建由 `build:prod` 复制到根目录 `static/` 后交给后端托管。
- `tests/`: 后端契约、runtime、support 与 infra 分层。
- `deploy/`: 生产发布与本地 Docker 调试资产真源。
- `config/`: `default` / `development` / `production` 分层配置。
- `memory/`: 当前项目长期记忆、边界和真源摘要。

## 局部 AGENTS

- `src/AGENTS.md`: 后端入口、分层和共享运行时资源约束。
- `src/modules/AGENTS.md`: 模块化迁移边界与 `api / application / domain / infrastructure` 规则。
- `frontend/src/AGENTS.md`: 前端启动骨架、HTTP 真源和验证要求。
- `frontend/src/features/AGENTS.md`: feature 层的 `api / model / ui / index.ts` 约定。
- `frontend/src/entities/AGENTS.md`: entity 层的单域网关、公共出口与禁用依赖。

## 日常命令

- 启动后端：`cargo run`
- 运行迁移：`cargo run --bin migrate`
- 后端源码内测试：`cargo test --lib`
- 认证契约测试：`cargo test --test auth_integration_test`
- 验证码发送契约测试：`cargo test --test send_verification_code_integration_test`
- release readiness 测试：`cargo test --test release_readiness_test`
- 前端行为测试：`pnpm --dir frontend test`
- 前端 lint：`pnpm --dir frontend lint`
- 前端生产构建：`pnpm --dir frontend build:prod`
- 架构守护：`powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`
- 部署文件改动后的 Compose 自检：`docker compose -f deploy/docker-compose.local.yml config`

## 稳定约束

- 后端是 Rust + Axum 单体服务，前端是 React + Vite；运行时依赖 PostgreSQL 与 Redis。
- 配置加载顺序固定为：`config/default.toml` -> `config/{APP_ENV}.toml` -> `APP__<SECTION>__<KEY>` 环境变量覆盖；`APP_ENV` 默认是 `development`。
- `development` 环境只能连接本地 PostgreSQL / Redis；不要把远端开发库地址写回开发配置。
- 不要启用全局 `try_parsing(true)`；当前配置链路依赖保留前导零字符串。
- 生产敏感配置必须走 `APP__...__..._FILE` 链路，例如 `APP__DATABASE__PASSWORD_FILE`、`APP__JWT__SECRET_FILE`、`APP__QQ_BOT__BEARER_TOKEN_FILE`。
- 生产发布唯一主线是 `.github/workflows/docker-build.yml` + `deploy/`；`deploy/build.sh` 与 `deploy/docker-compose.local.yml` 只用于本地 Docker 调试。
- `deploy/smoke.targets` 是本地 readiness test 与 release smoke 共用的检查目标真源；改健康检查路径、静态入口或产物文件名时，测试、脚本和文档必须一起更新。
- 前端真实 HTTP client 真源是 `frontend/src/shared/api/http-client.ts`；`frontend/src/services/api.ts` 只保留兼容 facade。
- 认证集成测试必须走真实 `/api/auth/verify-and-login` 链路；不要回退到本地签发 JWT 伪造主路径。
- `static/` 只保留目录占位 `.gitkeep`，不要把构建产物重新纳入版本控制。

## 变更同步要求

- 修改部署链路时，同步更新 `README.md`、`docs/guides/DOCKER_DEPLOYMENT.md`、`docs/INDEX.md`、`deploy/README.md`、`memory/03-deploy-and-risk-memory.md`。
- 修改仓库结构或目录职责时，同步更新 `memory/01-repo-shape.md`。
- 修改运行时、secrets 或环境变量链路时，同步更新 `memory/02-runtime-and-config.md`。
- 修改鉴权、缓存、handler/usecase 边界时，同步更新 `memory/06-maintainability-seams.md`。
- 修改测试入口、测试 support、CI 门禁或 smoke/readiness 契约时，同步更新 `docs/guides/TESTING.md`、`.github/workflows/ci.yml`、`deploy/smoke.targets`、`local-temp/测试代码优化方案/04-推荐实施计划路径.md`、`memory/07-testing-and-quality-gates.md`。
- 更新文档时，明确区分“生产发布主线”和“本地 Docker 调试路径”；不要把两者混写。
- 发现 `docs/README.md` 或 `docs/INDEX.md` 与当前代码、workflow 漂移时，优先修正文档。

## 验证要求

- 只改文档或 agent 指令时，至少做路径、命令、引用和真源一致性自检。
- 涉及后端行为改动时，按影响范围运行：`cargo test --lib`、`cargo test --test auth_integration_test`、`cargo test --test send_verification_code_integration_test`、`cargo test --test release_readiness_test`。
- 涉及前端结构、测试或发布链路改动时，至少运行：`pnpm --dir frontend test`、`pnpm --dir frontend lint`、`pnpm --dir frontend build:prod`。
- 涉及架构边界调整时，运行 `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`。
- 涉及部署相关改动时，至少做路径/引用扫描；Docker 可用时再跑 `docker compose -f deploy/docker-compose.local.yml config`。

## 参考资料

- 文档索引：`docs/INDEX.md`
- 测试真源：`docs/guides/TESTING.md`
- 部署说明：`deploy/README.md`
- 项目长期记忆：`memory/*.md`
