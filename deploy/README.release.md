# Electricity Monitor 发布包说明

此发布包由 GitHub Actions 生成，目标运行环境为 Alibaba Cloud Linux 3 amd64。

镜像在工作流中基于 `config/production.toml.example -> config/production.toml` 的运行时配置构建；生产数据库密码、JWT secret、QQ bot token 和 SMTP 授权码仍以运行时 secret file 覆盖为准。CORS、QQ 机器人发送地址、机器人 QQ、公开站点域名/端口和管理员 QQ 通过 `.env` 注入，不写入模板。

app release 包只离线携带应用镜像，服务器部署时只执行 `docker load`，不会从外部 registry 拉取镜像。PostgreSQL 和 Redis 镜像由独立的 infra images artifact 提供；首次部署、服务器缺少基础镜像，或基础镜像版本变更时，需要先把 infra 包解压到同一个 release 父目录。

## 包内内容

- `images/`
  - `electricity-monitor-__RELEASE_TAG__-linux-amd64.tar.gz`
- `compose.yaml`
- `deploy.sh`
- `smoke.sh`
- `smoke.targets`
- `.env.example`
- `release-manifest.json`

## 部署步骤

1. 解压 `release-__RELEASE_TAG__.tar.gz`。
2. 首次部署或基础镜像变更时，将 `infra-images-__RELEASE_TAG__.tar.gz` 解压到同一个 release 父目录，让其中的 `release/images/postgres-16-alpine-linux-amd64.tar.gz` 与 `release/images/redis-8-alpine-linux-amd64.tar.gz` 合并进当前 release 目录。若服务器已存在 `.env` 中声明的 PostgreSQL / Redis 镜像，可跳过此步。
3. 将 `.env.example` 复制为 `.env`。
4. 按 `.env` 中的约定填写必需运行时值，并在 `./secrets/` 下准备对应的 secret 文件，包括 `app_email_smtp_password`，把权限收紧到仅 owner 可读写，例如 `chmod 600 ./secrets/*`。如果需要跨 release 保留数据，建议把 `POSTGRES_DATA_DIR` 与 `REDIS_DATA_DIR` 改成 release 目录之外的稳定路径。部署时 `deploy.sh` 会把 secret owner 切到 `APP_RUNTIME_UID/GID`，保证非 root 应用进程能读取 Compose file secret。
5. 运行以下命令：

```bash
chmod +x deploy.sh
./deploy.sh
./smoke.sh
```

部署完成后，脚本会在当前 release 目录内写出 `deploy-result.json`，用于记录已部署的 tag、SHA 和镜像摘要。
`deploy.sh` 会在真正启动容器前校验 secret file 是否存在且权限已收紧到仅 owner 可读写。
`deploy.sh` 还会在 `docker compose up` 前校验 manifest 声明的归档 SHA256、加载后的镜像摘要，并检查应用、PostgreSQL 和 Redis 镜像是否已离线可用；缺少 infra 镜像时会直接失败，不会让服务器尝试从外部 registry 拉取镜像。
日常发布默认不重建 PostgreSQL / Redis；如果基础服务镜像摘要发生变化，脚本会 fail-fast。只有确认数据库备份和停机窗口后，才应设置 `DEPLOY_RECREATE_BASE_SERVICES=true` 重建基础服务容器。
`smoke.sh` 会读取 `smoke.targets`，因此 smoke 检查目标、必需文件和统一响应安全头都与本地 readiness test 保持同一份契约真源。

## 运行契约

- 运行方式：`docker compose`
- 服务：`postgres` + `redis` + 一次性 `migrate` + `app`
- 重启策略：`unless-stopped`
- 健康检查：`GET /api/health`
- 默认应用日志级别：`warn`
- 稳定容器名：
  - `electricity-app`
  - `electricity-postgres`
  - `electricity-redis`
- 默认端口绑定：`127.0.0.1:11450 -> app:8000`
- 数据目录：`.env` 中的 `POSTGRES_DATA_DIR` 与 `REDIS_DATA_DIR`。`deploy.sh` 会按 `POSTGRES_DATA_UID/GID` 与 `REDIS_DATA_UID/GID` 修正目录属主，避免 bind mount 目录由 root 创建后容器无法初始化。

## 回滚行为

如果部署后的健康检查失败，`deploy.sh` 会：

1. 停止并移除新启动的应用容器。
2. 重新用旧应用镜像标签启动 `electricity-app`。
3. 保持 PostgreSQL 和 Redis 容器原地运行，不再通过 rename 备份它们。

旧镜像还会额外打上 `rollback-<timestamp>` 标签，便于后续排查和人工确认。容器级回滚不会自动反向执行已经成功落库的迁移；涉及破坏性 schema 变更时，发布前应先准备数据库备份和人工回滚步骤。
