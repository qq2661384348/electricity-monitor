# Docker 部署指南

本文档描述当前推荐的发布链路：由 GitHub Actions 在 CI 中构建镜像、导出发布包、上传为 artifact，再由服务器执行一键部署脚本完成上线。

## 当前发布链路

### 目标

- 不在personal development environment上构建生产镜像
- 不在服务器上从源码重新编译
- 服务器只负责 `docker load`、`docker compose up`、健康检查与失败回滚

### 发布产物

手动触发工作流后，会生成一个 GitHub Actions artifact：

```text
release-<git-tag>.tar.gz
└── release/
    ├── images/
    │   ├── electricity-monitor-<git-tag>-linux-amd64.tar.gz
    │   └── redis-8-alpine-linux-amd64.tar.gz
    ├── compose.yaml
    ├── deploy.sh
    ├── smoke.sh
    ├── smoke.targets
    ├── .env.example
    ├── release-manifest.json
    └── README.md
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
3. 使用 `deploy/Dockerfile` 在 GitHub Actions Linux runner 中构建 `linux/amd64` Docker 镜像
4. 导出应用镜像与 Redis 镜像
5. 复制 `deploy/smoke.targets` 作为 release smoke 契约文件
6. 生成 `release-manifest.json`，写入 tag、git SHA、镜像 digest 与归档校验值
7. 组装 release 目录并压缩成单个归档
8. 上传为 GitHub Actions artifact

`static/` 是前端构建产物目录，不再作为仓库真源提交；CI 在 `pnpm build:prod` 后生成它，再由 `deploy/Dockerfile` 复制进入镜像。

### 构建性能优化

当前工作流已针对构建链路做了以下优化：

- 前端依赖通过 `actions/setup-node` 的 `pnpm` 缓存复用
- 前端安装使用 `pnpm install --frozen-lockfile`
- Docker 镜像构建使用 `docker/build-push-action` + `gha` 缓存
- `deploy/Dockerfile` 继续复用多阶段构建与 `cargo-chef`
- `deploy/Dockerfile.dockerignore` 用于约束镜像构建上下文
- CI 直接构建 `linux/amd64` 镜像，避免本地与线上重复构建

## 服务器部署

### 1. 下载 artifact

在 GitHub Actions 页面下载对应 tag 的 artifact，并上传到服务器。

### 2. 解压发布包

```bash
mkdir -p /opt/electricity-monitor
tar -xzf release-<git-tag>.tar.gz -C /opt/electricity-monitor
cd /opt/electricity-monitor/release
```

### 3. 配置环境变量

```bash
cp .env.example .env
vim .env
```

至少需要修改：

- `APP_DATABASE_PASSWORD_SECRET_FILE`
- `APP_JWT_SECRET_SECRET_FILE`
- `APP_QQ_BOT_BEARER_TOKEN_SECRET_FILE`

可按需覆盖：

- `APP__DATABASE__HOST`
- `APP__DATABASE__PORT`
- `APP__DATABASE__USERNAME`
- `APP__DATABASE__DATABASE`
- `APP_HOST_PORT`
- `APP_BIND_ADDRESS`

### 4. 执行一键部署

```bash
chmod +x deploy.sh
./deploy.sh
```

`deploy.sh` 会自动完成：

1. 校验 `docker` / `docker compose` / `gzip` / `curl`
2. 读取 `release-manifest.json` 并校验 `APP_IMAGE_REF`
3. 加载 `images/` 下的镜像归档
4. 备份现有 `electricity-app` / `electricity-redis`
5. 使用 `compose.yaml` 启动新版本
6. 对 `GET /api/health` 做重试健康检查
7. 将本次部署结果写入 `deploy-result.json`
8. 若健康检查失败，则自动回滚到旧容器

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

本地 `tests/release_readiness_test.rs` 与 release 包内 `smoke.sh` 读取同一份 `smoke.targets`，避免两边各自硬编码检查目标。

## 运行时约定

### 编排方式

- 运行方式：`docker compose`
- 服务：`app` + `redis`
- 重启策略：`unless-stopped`

### 容器身份

- 应用容器：`electricity-app`
- Redis 容器：`electricity-redis`

### 端口

- 容器端口：`8000`
- 默认宿主机端口：`11450`
- 默认绑定地址：`127.0.0.1`

### 健康检查

- 容器内健康检查：`http://localhost:8000/api/health`
- 部署成功判定：`.env` 中的 `DEPLOY_HEALTHCHECK_URL`
- 默认值：`http://127.0.0.1:11450/api/health`

## 回滚机制

部署脚本采用容器级回滚：

1. 发现旧容器后，先为当前镜像打 `rollback-<timestamp>` 标签
2. 将旧容器重命名为 `*-backup-<timestamp>`
3. 启动新容器并执行健康检查
4. 若失败：
   - 删除新容器
   - 将备份容器重命名回原名称
   - 重新启动旧容器

这意味着脚本不会只“报错退出”，而是会尝试恢复到上一个可运行状态。

## 发布身份记录

- `release-manifest.json`：由 GitHub Actions 生成，记录 `git_tag`、`git_sha`、镜像 digest、归档 SHA256、前端静态资源校验值。
- `deploy-result.json`：由服务器侧 `deploy.sh` 生成，记录本次部署状态、使用的 manifest 身份信息和健康检查目标。

## 本地调试说明

仓库中的 `deploy/build.sh` 和 `deploy/docker-compose.local.yml` 仍可用于本地 Docker 调试，但它们不再是推荐的生产发布主线。

生产发布主线以 GitHub Actions artifact 为准。
