# Docker 部署指南

本文档描述当前推荐的发布链路：由 GitHub Actions 在 CI 中构建镜像、导出发布包、上传为 artifact，再由服务器执行一键部署脚本完成上线。

## 当前发布链路

### 目标

- 不在个人开发环境中构建生产镜像
- 不在服务器上从源码重新编译
- 服务器只负责归档 SHA256 校验、`docker load`、镜像摘要校验、镜像离线可用性检查、`docker compose up`、健康检查与失败回滚

### 发布产物

手动触发工作流后，会生成两个 GitHub Actions artifact。日常发布只需要下载 app release artifact；首次部署、服务器缺少基础镜像，或 PostgreSQL / Redis 镜像版本变更时，再下载 infra images artifact。

app release artifact：

```text
release-<git-tag>.tar.gz
└── release/
    ├── images/
    │   └── electricity-monitor-<git-tag>-linux-amd64.tar.gz
    ├── compose.yaml
    ├── deploy.sh
    ├── smoke.sh
    ├── smoke.targets
    ├── .env.example
    ├── release-manifest.json
    └── README.md
```

infra images artifact：

```text
infra-images-<git-tag>.tar.gz
└── release/
    ├── images/
    │   ├── postgres-16-alpine-linux-amd64.tar.gz
    │   └── redis-8-alpine-linux-amd64.tar.gz
    └── infra-manifest.json
```

## 仓库内部署文件布局

生产发布与本地 Docker 调试的相关文件现在都集中在仓库 `deploy/` 目录：

```text
deploy/
├── Dockerfile
├── Dockerfile.dockerignore
├── build.sh
├── docker-compose.local.yml
├── compose.release.yml
├── deploy.sh
├── release.env.example
└── README.release.md
```

- `deploy/Dockerfile` + `deploy/Dockerfile.dockerignore`：GitHub Actions 构建镜像的真源
- `deploy/compose.release.yml` / `deploy/release.env.example` / `deploy/deploy.sh` / `deploy/README.release.md`：release 包模板
- `deploy/smoke.targets`：本地 readiness test 与 release smoke 共用的检查契约
- `deploy/build.sh` / `deploy/docker-compose.local.yml`：仅用于本地 Docker 调试，不是生产发布主线

## GitHub Actions 工作流

工作流文件：`.github/workflows/docker-build.yml`

### 触发方式

- 仅支持 `workflow_dispatch`
- 必须输入 `git_tag`

### 工作流行为

1. 校验 tag 是否存在，并检出该 tag
2. 构建前端并复制到 `static/`
3. 将 `config/production.toml.example` 复制为工作区内的 `config/production.toml`
4. 使用 `deploy/Dockerfile` 在 GitHub Actions Linux runner 中构建 `linux/amd64` Docker 镜像
5. 导出应用、PostgreSQL 和 Redis 镜像，保证服务器无需从外部 registry 拉取运行镜像
6. 复制 `deploy/smoke.targets` 作为 release smoke 契约文件
7. 生成 `release-manifest.json`，写入 tag、git SHA、镜像 digest、归档文件名与归档 SHA256
8. 组装 app release 包，只携带应用镜像和部署脚本
9. 组装 infra images 包，只携带 PostgreSQL / Redis 镜像
10. 上传为两个 GitHub Actions artifact：
   - `electricity-monitor-app-release-<git-tag>`
   - `electricity-monitor-infra-images-<git-tag>`

`static/` 是前端构建产物目录，不再作为仓库真源提交；CI 在 `bun run build:prod` 后生成它，再由 `deploy/Dockerfile` 复制进入镜像。
`deploy/Dockerfile` 会在构建期检查 `config/` 下是否只保留一个运行时 TOML，且文件名只能是 `development.toml` 或 `production.toml`，避免镜像带着歧义配置构建成功。
`config/production.toml.example` 中与浏览器访问和管理员权限直接相关的字段也必须在发布前改成真实值，尤其是：

- `cors.allowed_origins`
- `auth.refresh_cookie_secure`
- `auth.refresh_cookie_same_site`
- `qq_bot.api_url`
- `qq_bot.public_qq_number`
- `public_site.domain`
- `public_site.port`
- `admin.default_qq_number`

### 构建性能优化

当前工作流已针对构建链路做了以下优化：

- 前端工具链通过 `oven-sh/setup-bun@v2` 安装，并从 `frontend/package.json` 的 `packageManager` 读取 Bun 版本
- 前端安装使用 `bun install --frozen-lockfile`
- Docker 镜像构建使用 `docker/build-push-action` + `gha` 缓存
- `deploy/Dockerfile` 继续复用多阶段构建与 `cargo-chef`
- `deploy/Dockerfile.dockerignore` 用于约束镜像构建上下文
- CI 直接构建 `linux/amd64` 镜像，避免本地与线上重复构建

## Artifact 部署契约

公开文档只描述可复用的 release artifact 部署契约。环境专用上传自动化、SSH host alias、固定服务器目录和从开发配置派生生产 secret 的流程不属于仓库公开真源，应保存在仓库外或 git ignored runbook 中。

通用部署流程为：

1. 触发 `.github/workflows/docker-build.yml`，并指定 `git_tag`。
2. 下载 `electricity-monitor-app-release-<git-tag>`；首次部署或基础镜像变更时，再下载 `electricity-monitor-infra-images-<git-tag>`。
3. 将 artifact 上传到 `<server>` 的 `<release-root>/<git-tag>`。
4. 在服务器解压 app release 包；如需 infra images，将 infra 包解压到同一个 release 父目录以合并 `release/images/`。
5. 在服务器从 `.env.example` 准备 `.env`，并在 `secrets/` 下创建数据库、JWT、QQ token 和 SMTP 授权码 secret files。生产 secret 应来自部署控制面或运维密钥源，不从开发环境配置派生。
6. 执行 release 包内 `deploy.sh` 和 `smoke.sh`。
7. 如部署环境需要固定当前版本指针，由外部发布流程维护 `<current-release-symlink>`。

## 手动服务器部署

### 1. 下载 artifact

在 GitHub Actions 页面下载对应 tag 的 `electricity-monitor-app-release-<git-tag>` artifact，并上传到服务器。

如果服务器还没有 `postgres:16-alpine` 或 `redis:8-alpine`，或者本次变更升级了基础镜像版本，同时下载 `electricity-monitor-infra-images-<git-tag>` artifact。

### 2. 解压发布包

```bash
mkdir -p <release-root>/<git-tag>
tar -xzf release-<git-tag>.tar.gz -C <release-root>/<git-tag>
cd <release-root>/<git-tag>/release
```

如果需要加载 infra 包，先在同一个 release 父目录解压：

```bash
tar -xzf infra-images-<git-tag>.tar.gz -C <release-root>/<git-tag>
```

该命令会把 PostgreSQL / Redis 镜像归档合并到 `release/images/`。日常 app-only 发布可以跳过这一步，前提是服务器已经存在 `.env` 中声明的 `POSTGRES_IMAGE_REF` 和 `REDIS_IMAGE_REF`。

### 3. 配置环境变量

```bash
cp .env.example .env
vim .env
```

至少需要修改：

- `APP_DATABASE_PASSWORD_SECRET_FILE`
- `APP_JWT_SECRET_SECRET_FILE`
- `APP_QQ_BOT_BEARER_TOKEN_SECRET_FILE`
- `APP_EMAIL_SMTP_PASSWORD_SECRET_FILE`
- `APP__CORS__ALLOWED_ORIGINS`
- `APP__QQ_BOT__API_URL`
- `APP__QQ_BOT__PUBLIC_QQ_NUMBER`
- `APP__PUBLIC_SITE__DOMAIN`
- `APP__PUBLIC_SITE__PORT`
- `APP__ADMIN__DEFAULT_QQ_NUMBER`

对应的宿主机 secret 文件必须在部署前收紧到仅 owner 可读写，例如 `chmod 600 ./secrets/*`。SMTP 授权码使用 `APP_EMAIL_SMTP_PASSWORD_SECRET_FILE` 指向的宿主机文件提供，容器内固定挂载为 `/run/secrets/app_email_smtp_password`。`deploy.sh` 会把 secret owner 切到 `APP_RUNTIME_UID/GID`，因为应用镜像以非 root 用户运行，不能读取 root-only 的 Compose file secret。
如果使用稳定 release 目录之外的数据路径，设置：

- `POSTGRES_DATA_DIR=<data-root>/postgres`
- `REDIS_DATA_DIR=<data-root>/redis`

可按需覆盖：

- `APP__DATABASE__USERNAME`
- `APP__DATABASE__DATABASE`
- `POSTGRES_USER`
- `POSTGRES_DB`
- `APP_HOST_PORT`
- `APP_BIND_ADDRESS`

应用侧会直接追加统一响应安全头，并负责 CORS 白名单与 refresh cookie 行为；反向代理、TLS、`Strict-Transport-Security` 与 WAF 仍由部署环境负责。

### 4. 执行一键部署

```bash
chmod +x deploy.sh
./deploy.sh
```

`deploy.sh` 会自动完成：

1. 校验 `docker` / `docker compose` / `gzip` / `curl`
2. 校验 `.env` 中声明的 secret file 存在且权限已收紧到仅 owner 可读写
3. 读取 `release-manifest.json` 并校验 `APP_IMAGE_REF`
4. 校验 manifest 声明的归档 SHA256，拒绝未知归档、缺失归档或摘要不一致的 release 包
5. 加载 `images/` 下的镜像归档，并校验加载后的 app / PostgreSQL / Redis 镜像摘要
6. 检查 `APP_IMAGE_REF`、`POSTGRES_IMAGE_REF` 与 `REDIS_IMAGE_REF` 对应镜像是否已离线可用；缺失时直接失败，不触发外部 registry 拉取
7. 为当前应用镜像打 `rollback-<timestamp>` 标签
8. 使用 `compose.yaml` 启动 PostgreSQL 和 Redis；日常发布默认不重建基础服务，基础镜像摘要变化时必须显式设置 `DEPLOY_RECREATE_BASE_SERVICES=true`
9. 通过应用镜像中的内嵌 `migrate` 二进制执行数据库迁移
10. 启动新版本应用容器
11. 对 `GET /api/health` 做重试健康检查
12. 将本次部署结果写入 `deploy-result.json`
13. 若启动、迁移或健康检查失败，则自动回滚容器

### 5. 执行 smoke 检查

```bash
chmod +x smoke.sh
./smoke.sh
```

`smoke.sh` 会校验：

- `smoke.targets` 中声明的 `/api/health`
- `smoke.targets` 中声明的 `/api/health/db`
- `smoke.targets` 中声明的静态入口 `/`
- `smoke.targets` 中声明的 `release-manifest.json`
- `smoke.targets` 中声明的 `deploy-result.json`
- `smoke.targets` 中声明的统一响应安全头

本地 `tests/runtime/release_readiness_test.rs` 与 release 包内 `smoke.sh` 读取同一份 `smoke.targets`，避免两边各自硬编码检查目标。如果 `.env` 显式覆盖 `APP__CAPTCHA__API_URL`，`smoke.sh` 会按该 URL 派生 CSP 的验证码 origin 期望值。

## 运行时约定

### 编排方式

- 运行方式：`docker compose`
- 服务：`postgres` + `redis` + 一次性 `migrate` + `app`
- 重启策略：`unless-stopped`

### 容器身份

- 应用容器：`electricity-app`
- PostgreSQL 容器：`electricity-postgres`
- Redis 容器：`electricity-redis`

### 日志级别

- release 默认应用日志级别：`warn`
- 临时排障时可以通过 `.env` 中的 `APP__LOGGING__LEVEL` 改为 `info`、`debug` 或其他 tracing 级别；排障结束后应恢复为 `warn`，避免后台任务和轮询路径在公网服务器持续输出大量日志。

### 内存释放

- release 默认设置 `MIMALLOC_PURGE_DELAY=0` 与 `MIMALLOC_PURGE_DECOMMITS=1`，让 mimalloc 在后台批处理释放对象后尽快把空闲物理页交还给宿主机。
- 电费全量抓取必须保持流式背压和定时任务防重入；不要改回“一次性为所有房间创建 Tokio task”的实现，否则公网服务器上的容器 RSS 会被批处理高水位放大。

### 端口

- 容器端口：`8000`
- 默认宿主机端口：`11450`
- 默认绑定地址：`127.0.0.1`

### 数据目录

- PostgreSQL：`.env` 中的 `POSTGRES_DATA_DIR`
- Redis：`.env` 中的 `REDIS_DATA_DIR`
- 生产部署建议把数据目录放在 release 目录之外的稳定 `<data-root>` 下，例如 `<data-root>/postgres` 与 `<data-root>/redis`
- `deploy.sh` 会按 `.env` 中的 `POSTGRES_DATA_UID/GID` 与 `REDIS_DATA_UID/GID` 修正 bind mount 目录属主，避免 root 创建的数据目录导致 PostgreSQL 或 Redis 容器无法初始化。

### 健康检查

- 容器内健康检查：`http://localhost:8000/api/health`
- 部署成功判定：`.env` 中的 `DEPLOY_HEALTHCHECK_URL`
- 默认值：`http://127.0.0.1:11450/api/health`

## 回滚机制

部署脚本采用应用容器级回滚：

1. 发现旧应用容器后，先为当前应用镜像打 `rollback-<timestamp>` 标签
2. 使用 Compose 原地启动 PostgreSQL / Redis，不再 rename 这些依赖容器
3. 执行内嵌迁移，再启动应用容器并执行健康检查
4. 若失败：
   - 删除或替换新应用容器
   - 用旧应用镜像标签重新启动 `electricity-app`
   - 保持 PostgreSQL / Redis 稳定容器名不变

这意味着脚本不会只“报错退出”，而是会尝试恢复到上一个可运行状态。
容器回滚不等于数据库 schema 自动回滚；如果某次 release 包含破坏性迁移，发布前必须额外准备数据库备份和人工恢复步骤。

## 发布身份记录

- `release-manifest.json`：由 GitHub Actions 生成，记录 `git_tag`、`git_sha`、app 镜像 digest、app 归档文件名与 SHA256、PostgreSQL / Redis 镜像身份、infra artifact 名称、infra 归档文件名与 SHA256 和前端静态资源校验值。服务器侧 `deploy.sh` 会据此校验归档完整性和加载后的镜像摘要。
- `deploy-result.json`：由服务器侧 `deploy.sh` 生成，记录本次部署状态、使用的 manifest 身份信息和健康检查目标。

## 本地调试说明

仓库中的 `deploy/build.sh` 和 `deploy/docker-compose.local.yml` 仍可用于本地 Docker 调试，但它们不再是推荐的生产发布主线。

`deploy/build.sh` 在检测到缺少 `config/development.toml` 时，会自动从 `config/development.toml.example` 复制一份本地运行时配置；本地 Compose 会覆盖数据库、Redis 和必要公开运行时值。离开 Compose 直接运行后端时，仍需按模板补齐对应开发配置。

生产发布主线以 GitHub Actions artifact 为准。
