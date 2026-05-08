# 部署资产说明

`deploy/` 是仓库内部与部署相关文件的统一入口。

## 生产发布资产

- `Dockerfile`: GitHub Actions 构建 release 镜像时使用的 Dockerfile
- `Dockerfile.dockerignore`: 与 `Dockerfile` 配套的构建忽略规则
- `compose.release.yml`: release 包内的 `compose.yaml` 模板
- `deploy.sh`: release 包内的一键部署脚本模板
- `smoke.sh`: release 包内的部署后 smoke 检查脚本模板
- `smoke.targets`: readiness / smoke 共用的检查目标定义
- `release-manifest.json`: GitHub Actions 组装 artifact 时生成的版本清单
- `release.env.example`: release 包内的 `.env.example` 模板
- `README.release.md`: release 包内的 `README.md` 模板

## 本地 Docker 调试

- `build.sh`: 本地 Docker 调试入口
- `docker-compose.local.yml`: 本地 Docker 调试使用的 compose 文件
- `build.sh` 会在缺少 `config/development.toml` 时自动从 `config/development.toml.example` 复制一份本地运行时配置；本地 Compose 会覆盖数据库、Redis 和必要公开运行时值。离开 Compose 直接运行后端时，仍需按模板补齐对应开发配置。如需本地调试邮件发送，再通过 `APP__EMAIL__SMTP_PASSWORD(_FILE)` 注入 SMTP 授权码。

## 当前真源

- 生产发布主线：`.github/workflows/docker-build.yml`
- PR / 手动质量门禁：`.github/workflows/ci.yml`
- 服务器部署方式：触发 GitHub Actions 生成 release artifact，上传到 `<server>` 的 `<release-root>/<tag>`，再在服务器执行包内 `deploy.sh` / `smoke.sh`
- GitHub Actions 会拆分产出 app release artifact 与 infra images artifact；日常发布只需要 app 包，首次部署或 PostgreSQL / Redis 镜像版本变更时再把 infra 包解压到同一 release 目录
- 服务器只执行 `docker load`，不从外部 registry 拉取镜像；`deploy.sh` 会在启动前校验 release manifest 中声明的归档 SHA256、加载后的镜像摘要，以及 app、PostgreSQL 和 Redis 镜像是否已离线可用
- release compose 服务包含 `postgres`、`redis`、一次性 `migrate` 和 `app`；`deploy.sh` 会先启动依赖、执行内嵌数据库迁移，再启动应用
- 日常发布默认不重建 PostgreSQL / Redis；如果基础服务镜像摘要发生变化，脚本会 fail-fast，只有显式设置 `DEPLOY_RECREATE_BASE_SERVICES=true` 才会在已确认备份和停机窗口后重建基础服务容器
- 默认对外绑定为 `127.0.0.1:11450 -> app:8000`，不包含反向代理配置
- release 默认应用日志级别为 `warn`，避免公网服务器在后台任务和轮询场景下输出大量 `info` 日志；需要临时排障时再通过 `.env` 中的 `APP__LOGGING__LEVEL` 提高日志详细度
- release 默认启用 `MIMALLOC_PURGE_DELAY=0` 和 `MIMALLOC_PURGE_DECOMMITS=1`，让应用容器在全量电费抓取等批处理后更及时归还空闲物理页
- release 解压根目录、当前版本指针和持久数据目录由部署环境决定；公开契约统一使用 `<release-root>`、`<current-release-symlink>` 与 `<data-root>` 占位，`.env` 中的 `POSTGRES_DATA_DIR` / `REDIS_DATA_DIR` 指向实际数据目录
- 本地 Docker 调试前会优先使用 `config/development.toml.example -> config/development.toml` 的本地运行时配置；Compose 会覆盖数据库、Redis 和必要公开运行时值，离开 Compose 直接运行后端时仍需按模板补齐对应开发配置
- 生产敏感配置通过 Compose secrets 提供，并由 `.env` 中的 `*_SECRET_FILE` 指向宿主机文件，当前覆盖数据库密码、JWT secret、QQ bearer token 和 SMTP 授权码
- 环境专用上传自动化不属于仓库公开真源；公开文档只维护 artifact 部署契约，不记录 SSH host alias、固定服务器路径或从开发配置派生生产 secret 的流程
- `.env` 还必须显式填写 CORS、QQ 机器人发送配置、公开站点域名/端口和管理员 QQ；这些值不会写入 `production.toml.example`
- release 部署前必须把 secret file 权限收紧到仅 owner 可读写；`deploy.sh` 会对权限过宽的文件直接 fail-fast
- 服务器部署时会读取 `release-manifest.json`，并在 release 目录写出 `deploy-result.json`
- `smoke.targets` 是本地 readiness test 与 release smoke 的共享契约真源，包含端点、必需文件与统一响应安全头；`.env` 显式覆盖 `APP__CAPTCHA__API_URL` 时，`smoke.sh` 会同步派生 CSP 验证码 origin 期望值
- 本目录中的本地调试脚本不改变生产发布主线，只作为开发/排障辅助
