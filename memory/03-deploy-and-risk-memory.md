# Electricity Monitor 仓库记忆：部署链路与长期风险

## 生产发布主线

- GitHub Actions 工作流真源是 `.github/workflows/docker-build.yml`。
- 镜像构建真源是 `deploy/Dockerfile`，release 模板真源是 `deploy/` 目录。
- 工作流通过 `workflow_dispatch` 接收 `git_tag`，输出 `release-<tag>.tar.gz`。
- release 包固定包含应用镜像、Redis 镜像、`compose.yaml`、`deploy.sh`、`smoke.sh`、`smoke.targets`、`.env.example`、`README.md` 和 `release-manifest.json`。

## 仓库内部署资产布局

- `deploy/compose.release.yml`、`deploy/release.env.example`、`deploy/deploy.sh`、`deploy/smoke.sh`、`deploy/smoke.targets`、`deploy/README.release.md` 负责 release 包模板。
- `deploy/build.sh` 与 `deploy/docker-compose.local.yml` 只用于本地 Docker 调试，不是生产发布主线。
- 根目录不再直接存放部署脚本或 Docker 编排文件，部署边界以 `deploy/` 为准。

## 构建与服务器职责

- CI 会先在 `frontend/` 中执行 `bun install --frozen-lockfile` 与 `bun run build:prod`，再把 `static/` 产物打入镜像。
- CI 在构建镜像前会把 `config/production.toml.example` 复制为工作区内的 `config/default.toml`。
- Docker 镜像构建继续使用 Buildx + GHA cache，`deploy/Dockerfile` 继续复用 `cargo-chef` 多阶段构建。
- 服务器不再从源码构建；服务器职责是加载镜像、校验 `release-manifest.json`、挂载 secret files、执行 `docker compose up`、做健康检查、写出 `deploy-result.json`，并在失败时回滚。

## 共享部署契约

- `deploy/smoke.targets` 是本地 readiness test 与 release smoke 的共享检查目标真源。
- release artifact 会携带 `release-manifest.json`；服务器部署结果写入 `deploy-result.json`。
- `deploy.sh` 会读取 manifest 做一致性校验，`smoke.sh` 会检查 `/api/health`、`/api/health/db`、静态入口和 manifest / result 文件。

## 长期风险

- 仓库曾出现过把真实敏感配置写入 git 历史的风险，因此真实 `QQ/JWT/DB` 凭据必须继续留在运行时配置或 secret file 中，不能回写仓库。
- 真实凭据轮换仍依赖外部 secret file 或部署目标访问权；在没有外部执行证据前，不应在仓库文档中写成“已完成轮换”。
- 部署脚本的回滚路径仍需要在真实 Linux Docker 主机上做端到端演练，本地 readiness test 不能替代这类验证。
- 发布链路、配置模板、memory 和部署文档容易产生漂移；改部署时必须同步更新 `README.md`、`docs/guides/DOCKER_DEPLOYMENT.md`、`deploy/README.md` 和相关 memory。
