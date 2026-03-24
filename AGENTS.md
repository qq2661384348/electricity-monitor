# Project AGENTS

## Repo Snapshot

- 后端是 Rust + Axum 单体服务，主代码在 `src/`。
- 前端是 React + Vite，目录在 `frontend/`，生产构建产物复制到根目录 `static/` 后由后端托管。
- 运行时依赖 PostgreSQL 与 Redis。
- 配置加载顺序固定为：`config/default.toml` -> `config/{APP_ENV}.toml` -> `APP__<SECTION>__<KEY>` 环境变量覆盖。
- 后端当前的模块化迁移入口已经存在于 `src/modules/`，已落地 `auth`、`room`、`room_sync` 三个模块样板。

## Deployment Truth

- 生产发布唯一主线是 `.github/workflows/docker-build.yml`。
- 仓库内部署相关文件统一位于 `deploy/`，不要再把 Dockerfile、compose、部署脚本放回根目录。
- `deploy/Dockerfile` 与 `deploy/Dockerfile.dockerignore` 是镜像构建真源。
- `deploy/compose.release.yml`、`deploy/release.env.example`、`deploy/deploy.sh`、`deploy/smoke.sh`、`deploy/README.release.md` 是 release 包模板。
- `deploy/build.sh` 与 `deploy/docker-compose.local.yml` 仅用于本地 Docker 调试，不是生产发布真源。
- 服务器部署契约是：消费 GitHub Actions artifact，执行 `docker load`、`docker compose up`、健康检查 `/api/health`，失败时由 `deploy.sh` 回滚。
- release artifact 会携带 `release-manifest.json`；服务器部署结果写入 `deploy-result.json`。
- 生产敏感配置通过 Compose secrets 提供，`.env` 只保留 `*_SECRET_FILE` 路径，不保留秘密原文。

## Runtime Constraints

- `development` 环境只能连接本地 PostgreSQL / Redis；不要把远端开发库地址写回开发配置。
- `APP_ENV` 默认是 `development`。
- 不要启用全局 `try_parsing(true)`；当前配置链路依赖保留前导零字符串。
- 生产环境的敏感配置必须通过 `APP__...__..._FILE` 链路注入，例如：
  - `APP__DATABASE__PASSWORD_FILE`
  - `APP__JWT__SECRET_FILE`
  - `APP__QQ_BOT__BEARER_TOKEN_FILE`
- 固定 `admin_token` 已移除；管理员通过 `config.admin.default_qq_number` 对应账号登录后获取 `admin` 角色 JWT。
- 前端真实 HTTP client 真源是 `frontend/src/shared/api/http-client.ts`，`frontend/src/services/api.ts` 仅保留兼容 facade。
- `static/` 只保留目录占位 `.gitkeep`，不要再把构建产物重新纳入版本控制。

## Change Hygiene

- 修改部署链路时，至少同步：`README.md`、`docs/guides/DOCKER_DEPLOYMENT.md`、`docs/INDEX.md`、`deploy/README.md`、`memory/03-deploy-and-risk-memory.md`。
- 修改仓库结构或目录职责时，同步更新 `memory/01-repo-shape.md`。
- 修改运行时、secrets 或环境变量链路时，同步更新 `memory/02-runtime-and-config.md`。
- 修改鉴权、缓存、handler/usecase 边界时，同步更新 `memory/06-maintainability-seams.md`。
- 更新文档时，明确区分“生产发布主线”和“本地 Docker 调试路径”，不要把两者混写。
- `docs/README.md` 和 `docs/INDEX.md` 只能保留与当前代码、workflow 一致的描述；发现漂移时优先修正。
- 继续做架构升级时，优先进入 `src/modules/*/application`，不要把新的复杂编排回流到 `handlers/`。

## Verification Focus

- 部署相关改动至少要做路径/引用扫描，并在 Docker 可用时跑 `docker compose -f deploy/docker-compose.local.yml config`。
- 前端或发布链路改动后，确认生产构建仍会先生成 `static/`，再进入 Docker 镜像构建。
- 架构相关改动后，跑 `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`。
- 鉴权相关改动后，至少跑 `cargo test --test auth_integration_test`。
- 发布 readiness 相关改动后，至少跑 `cargo test --test release_readiness_test`。
