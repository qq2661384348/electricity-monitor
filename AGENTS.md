# 项目协作 AGENTS

## 作用范围

- 根目录 `AGENTS.md` 定义全仓共享规则；进入更深目录工作时，还要同时遵守就近的子级 `AGENTS.md`。
- 开始任务前先读 `memory/README.md`，再按任务范围阅读对应主题目录。
- 代码、配置、CI、脚本和当前文档真源优先于历史说明；不要把失效流程重新写回仓库。

## 仓库地图

- `src/`：Rust + Axum 后端主代码；启动装配在 `src/bootstrap/`，共享运行时资源在 `src/state.rs`。
- `src/modules/`：后端模块化迁移主线；复杂编排优先进入 `src/modules/*/application`。
- `frontend/`：React 19 + Vite 8 前端工作区；`bun` 是唯一包管理真源，`bun.lock` 是唯一前端 lockfile。
- `tests/`：后端契约、runtime、support 与 infra 分层。
- `deploy/`：生产发布与本地 Docker 调试资产真源。
- `config/`：按环境命名的运行时配置与环境模板；`config/` 下只能保留一个运行时 `.toml` 文件。
- `memory/`：项目长期记忆与边界摘要；`memory/README.md` 是唯一入口。

## 局部 AGENTS

- `src/AGENTS.md`：后端入口、共享状态、运行时安全头和最小验证。
- `src/modules/AGENTS.md`：模块化迁移边界与 `api / application / domain / infrastructure` 规则。
- `frontend/AGENTS.md`：前端工作区级包管理、构建、测试与静态产物规则。
- `frontend/src/AGENTS.md`：前端启动骨架、HTTP 真源、会话模型和源码层验证要求。
- `frontend/src/features/AGENTS.md`：feature 层 `api / model / ui / index.ts` 约定。
- `frontend/src/entities/AGENTS.md`：entity 层单域网关、公共出口与禁用依赖。

## 日常命令

- 启动后端：`cargo run`
- 运行迁移：`cargo run --bin migrate`
- 后端源码内测试：`cargo test --lib`
- 认证契约测试：`cargo test --test auth_integration_test`
- 验证码发送契约测试：`cargo test --test send_verification_code_integration_test`
- release readiness 测试：`cargo test --test release_readiness_test`
- 后端 clippy 门禁：`cargo clippy --all-targets -- -D warnings`
- Rust 依赖审计：`cargo audit -q`
- 前端安装依赖：`bun install --cwd frontend --frozen-lockfile`
- 前端行为测试：`bun run --cwd frontend test`
- 前端 lint：`bun run --cwd frontend lint`
- 前端 bundle 预算检查：`bun run --cwd frontend check:bundle`
- 前端生产构建：`bun run --cwd frontend build:prod`
- 前端依赖审计：`bun audit --cwd frontend`
- 架构守护：`powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`
- 部署文件改动后的 Compose 自检：`docker compose -f deploy/docker-compose.local.yml config`

## 稳定约束

- 后端运行时依赖 PostgreSQL 与 Redis。
- 配置加载顺序固定为“当前环境对应的运行时 TOML 文件” -> `APP__<SECTION>__<KEY>`；`APP_ENV` 默认是 `development`。
- `config/` 下只能存在一个运行时 `.toml` 文件，文件名必须是 `development.toml` 或 `production.toml`，且要与 `APP_ENV` 保持一致；两个运行时文件都不纳入版本控制。
- 开发环境从 `config/development.toml.example` 复制为 `config/development.toml`，生产/发布环境从 `config/production.toml.example` 复制为 `config/production.toml`。
- `development` 环境只能连接本地 PostgreSQL / Redis；开发环境数据库密码必须直接写入复制出来的 `config/development.toml`。
- 不要启用全局 `try_parsing(true)`；当前配置链路依赖保留前导零字符串。
- 生产敏感配置必须走 `APP__...__..._FILE` 链路，例如 `APP__DATABASE__PASSWORD_FILE`、`APP__JWT__SECRET_FILE`、`APP__QQ_BOT__BEARER_TOKEN_FILE`。
- `cors.allowed_origins` 是浏览器访问白名单真源；生产环境不能保留 `localhost` 或模板占位值。
- refresh token 只允许通过 `Set-Cookie` / `Cookie` 往返；JSON 响应只返回 access token。
- `admin.default_qq_number` 只有在显式真实值时才会授予管理员权限；生产环境不能保留占位值。
- 生产发布主线是 `.github/workflows/docker-build.yml` + `deploy/`；`deploy/build.sh` 与 `deploy/docker-compose.local.yml` 只用于本地 Docker 调试。
- 后端统一响应安全头由应用层直接追加；`deploy/smoke.targets` 是本地 readiness test 与 release smoke 共用的端点、文件与响应头契约真源。
- 前端真实 HTTP client 真源是 `frontend/src/shared/api/http-client.ts`；`frontend/src/services/api.ts` 只保留兼容 facade。
- `frontend/package.json` 中的 `overrides` 是前端传递依赖安全修复真源；不要绕开 Bun 锁定修复版本。
- `frontend/scripts/check-bundle-budgets.ts` 是前端 JS chunk 预算真源；`vite.config.ts` 里的 warning 阈值只做粗粒度提示。
- 前端 `build:prod` 会在构建后把 `frontend/dist/` 复制到根目录 `static/`；`static/` 只保留目录占位 `.gitkeep`，不要把构建产物重新纳入版本控制。
- 短期记忆只允许作为例外落在对应 `memory/` 子目录下，文件名必须使用 `st-` 前缀并写明失效条件；不要把临时过程直接写进长期 memory。

## 变更同步要求

- 修改 `memory/` 结构或用途时，同时更新 `memory/README.md` 与 `memory/01-governance/02-repo-shape-and-agents.md`。
- 修改仓库结构、目录职责或 AGENTS 拓扑时，同步更新 `memory/01-governance/02-repo-shape-and-agents.md`。
- 修改运行时、环境变量或 secrets 链路时，同步更新 `memory/02-runtime/01-config-and-environments.md`。
- 修改鉴权、cookie 会话、CORS 或管理员提升规则时，同步更新 `docs/api/API_REFERENCE.md` 与 `memory/02-runtime/02-auth-session-and-cors.md`。
- 修改前端工具链、前端工作区规则或前端边界时，同步更新 `frontend/README.md`、`frontend/AGENTS.md`、`memory/03-architecture/02-frontend-architecture.md`、`memory/03-architecture/03-frontend-seams.md`。
- 修改后端维护接缝、模块边界或外部 HTTP / 缓存接缝时，同步更新 `memory/03-architecture/01-backend-seams.md`。
- 修改架构热点判断时，同步更新 `memory/03-architecture/04-hotspots.md`。
- 修改部署链路或 release 包契约时，同步更新 `README.md`、`docs/guides/DOCKER_DEPLOYMENT.md`、`docs/INDEX.md`、`deploy/README.md`、`memory/04-delivery/01-deploy-and-release.md`。
- 修改测试入口、测试 support、CI 门禁或 smoke/readiness 契约时，同步更新 `docs/guides/TESTING.md`、`.github/workflows/ci.yml`、`deploy/smoke.targets`、`memory/04-delivery/02-testing-and-quality-gates.md`。
- 修改质量风险、安全风险或依赖审计基线时，同步更新 `memory/05-risks/01-quality-and-security-risks.md`。

## 验证要求

- 只改文档或 agent 指令时，至少做路径、命令、引用和真源一致性自检。
- 涉及后端行为改动时，按影响范围运行后端测试矩阵。
- 涉及运行时安全头、release smoke 或部署脚本权限校验改动时，至少运行 `cargo test --test release_readiness_test` 与 `cargo audit -q`。
- 涉及前端结构、测试、包管理或发布链路改动时，至少运行 `bun run --cwd frontend test`、`bun run --cwd frontend lint`、`bun run --cwd frontend build:prod`。
- 涉及架构边界调整时，运行 `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`。
- 涉及部署相关改动时，至少做路径/引用扫描；Docker 可用时再跑 `docker compose -f deploy/docker-compose.local.yml config`。

## 参考资料

- 文档索引：`docs/INDEX.md`
- 测试真源：`docs/guides/TESTING.md`
- 部署说明：`deploy/README.md`
- 项目长期记忆：`memory/README.md`
