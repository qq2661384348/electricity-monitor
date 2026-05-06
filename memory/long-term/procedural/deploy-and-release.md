---
type: procedural
status: verified
scope: 部署与 release 流程
updated_at: 2026-05-07
verified_at: 2026-05-07
sources:
  - .gitignore
  - .github/workflows/docker-build.yml
  - deploy/compose.release.yml
  - deploy/deploy.sh
  - deploy/README.md
  - deploy/README.release.md
  - deploy/release.env.example
  - deploy/smoke.targets
summary: 生产发布主线、release 产物、local environment中转边界、服务器职责和共享部署契约
---

# Electricity Monitor 部署与 release 契约

## 适用场景

- 需要理解生产发布主线、release 产物组成和服务器职责时。
- 需要调整部署脚本、release 包模板或 smoke 契约时。

## 前置条件

- 生产发布真源以 `.github/workflows/docker-build.yml` 和 `deploy/` 目录为准。
- 服务器部署只消费 release artifact，不从源码重新构建。

## 标准步骤

1. 由 GitHub Actions 工作流通过 `workflow_dispatch` 接收 `git_tag`，输出 `release-<tag>.tar.gz`。
2. CI 在 `frontend/` 中执行 `bun install --frozen-lockfile` 与 `bun run build:prod`，再把 `static/` 产物打入镜像。
3. CI 在构建镜像前把 `config/production.toml.example` 复制为工作区内的 `config/production.toml`，并保持 `config/` 下只存在这一个运行时 TOML。
4. release 包固定包含应用镜像、PostgreSQL 镜像、Redis 镜像、`compose.yaml`、`deploy.sh`、`smoke.sh`、`smoke.targets`、`.env.example`、`README.md` 和 `release-manifest.json`。
5. 服务器加载镜像、校验 `release-manifest.json`、挂载 secret files、执行 `docker compose` 启动 PostgreSQL / Redis、运行一次性 `migrate`、启动应用、做健康检查、写出 `deploy-result.json`，并在失败时回滚容器。

## 共享部署契约

- `deploy/compose.release.yml`、`deploy/release.env.example`、`deploy/deploy.sh`、`deploy/smoke.sh`、`deploy/smoke.targets`、`deploy/README.release.md` 负责 release 包模板。
- `deploy/build.sh` 与 `deploy/docker-compose.local.yml` 只用于本地 Docker 调试，不是生产发布主线。
- out-of-repository deployment automation属于out-of-repository private file，不纳入仓库和公开发布；仓库 `.gitignore` 忽略 `deploy/relay-deploy*.sh`。
- artifact deployment默认使用 `ssh <server>` 上传到 `<release-root>`，release 版本目录为 `<release-root>/releases/<tag>`，持久数据目录为 `<release-root>/data/postgres` 与 `<release-root>/data/redis`。
- release 包离线携带应用、PostgreSQL 和 Redis 镜像；服务器只执行 `docker load`，不从外部 registry 拉取这些镜像。
- release compose 服务拓扑为 `postgres`、`redis`、一次性 `migrate` 和 `app`，默认端口绑定为 `127.0.0.1:11450 -> app:8000`，不包含反向代理配置。
- release `deploy.sh` 会显式设置 PostgreSQL / Redis bind mount 数据目录属主；local environment中转或 root shell 创建的 root-only 数据目录不能直接交给容器使用。
- `migrate` 二进制使用编译期内嵌 migrations，目标环境运行迁移不需要安装 `diesel_cli`。
- release 部署脚本会对 `APP_DATABASE_PASSWORD_SECRET_FILE`、`APP_JWT_SECRET_SECRET_FILE`、`APP_QQ_BOT_BEARER_TOKEN_SECRET_FILE`、`APP_EMAIL_SMTP_PASSWORD_SECRET_FILE` 做 owner-only 权限校验；group / other 有权限位时直接失败。
- release 部署脚本随后会把 secret owner 切到 `APP_RUNTIME_UID/GID`，因为应用镜像以非 root 用户运行，Docker Compose file secret 在本地 compose 模式下会保留宿主机文件权限。
- SMTP 授权码在 release 包内通过 `APP_EMAIL_SMTP_PASSWORD_SECRET_FILE` 指向宿主机文件，并由 compose 挂载为 `/run/secrets/app_email_smtp_password`，应用侧通过 `APP__EMAIL__SMTP_PASSWORD_FILE` 读取。
- release `.env` 必须显式提供 `APP__CORS__ALLOWED_ORIGINS`、`APP__QQ_BOT__API_URL`、`APP__QQ_BOT__PUBLIC_QQ_NUMBER`、`APP__PUBLIC_SITE__DOMAIN`、`APP__PUBLIC_SITE__PORT` 与 `APP__ADMIN__DEFAULT_QQ_NUMBER`；这些非敏感运行时值不写入生产配置模板。
- `deploy/smoke.targets` 是本地 readiness test 与 release smoke 的共享检查目标真源。
- `deploy.sh` 会读取 manifest 做一致性校验，`smoke.sh` 会检查 `/api/health`、`/api/health/db`、静态入口、必需响应头和 manifest / result 文件。
- 容器级回滚不会自动反向执行已经成功落库的 schema 迁移；包含破坏性迁移的 release 必须在发布前准备数据库备份与人工恢复步骤。

## 验证方式

- 只改部署文档或契约时，至少做路径、引用与 `deploy/smoke.targets` 一致性扫描。
- 涉及 release smoke、部署脚本或 secret file 权限校验改动时，至少运行 `cargo test --test release_readiness_test` 与相关 shell 语法检查。
- Docker 可用时，再运行 `docker compose -f deploy/docker-compose.local.yml config` 做本地 Compose 自检。

## 常见风险

- 本地 Docker 调试链路不能替代真实 Linux Docker 主机上的回滚与 smoke 演练。
- 服务器端 secret file 权限、TLS 与反向代理能力属于仓库外部控制面，不能在仓库里伪造“已完成”。
