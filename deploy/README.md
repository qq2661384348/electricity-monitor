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
- `build.sh` 会在缺少 `config/development.toml` 时自动从 `config/development.toml.example` 复制一份本地运行时配置；运行前仍需填写数据库密码、`qq_bot.api_url`、`qq_bot.public_qq_number`、`qq_bot.bearer_token`、`public_site.domain` 和 `public_site.port`。如需本地调试邮件发送，再通过 `APP__EMAIL__SMTP_PASSWORD(_FILE)` 注入 SMTP 授权码。

## 当前真源

- 生产发布主线：`.github/workflows/docker-build.yml`
- PR / 手动质量门禁：`.github/workflows/ci.yml`
- 服务器部署方式：local environment使用 `gh` 触发 GitHub Actions、下载 artifact，再通过 `ssh/scp` 上传到服务器并执行包内 `deploy.sh`
- GitHub Actions 会拆分产出 app release artifact 与 infra images artifact；日常发布只需要 app 包，首次部署或 PostgreSQL / Redis 镜像版本变更时再把 infra 包解压到同一 release 目录
- 服务器只执行 `docker load`，不从外部 registry 拉取镜像；`deploy.sh` 会在 `docker compose up` 前检查 app、PostgreSQL 和 Redis 镜像是否已离线可用
- release compose 服务包含 `postgres`、`redis`、一次性 `migrate` 和 `app`；`deploy.sh` 会先启动依赖、执行内嵌数据库迁移，再启动应用
- 默认对外绑定为 `127.0.0.1:11450 -> app:8000`，不包含反向代理配置
- release 默认应用日志级别为 `warn`，避免公网服务器在后台任务和轮询场景下输出大量 `info` 日志；需要临时排障时再通过 `.env` 中的 `APP__LOGGING__LEVEL` 提高日志详细度
- artifact deployment默认把持久数据放在 `<release-root>/data/postgres` 与 `<release-root>/data/redis`，release 版本放在 `<release-root>/releases/<tag>`，当前版本软链为 `<release-root>/current`
- 本地 Docker 调试前会优先使用 `config/development.toml.example -> config/development.toml` 的本地运行时配置，且该运行时配置必须补齐 `database.password`、`qq_bot.api_url`、`qq_bot.public_qq_number`、`qq_bot.bearer_token`、`public_site.domain` 与 `public_site.port`
- 生产敏感配置通过 Compose secrets 提供，并由 `.env` 中的 `*_SECRET_FILE` 指向宿主机文件，当前覆盖数据库密码、JWT secret、QQ bearer token 和 SMTP 授权码
- out-of-repository deployment automation不要纳入仓库；`.gitignore` 已忽略 `deploy/relay-deploy*.sh`，可用它们from the deployment environment `config/development.toml` 派生远端 `.env` 与 secret files
- `.env` 还必须显式填写 CORS、QQ 机器人发送配置、公开站点域名/端口和管理员 QQ；这些值不会写入 `production.toml.example`
- release 部署前必须把 secret file 权限收紧到仅 owner 可读写；`deploy.sh` 会对权限过宽的文件直接 fail-fast
- 服务器部署时会读取 `release-manifest.json`，并在 release 目录写出 `deploy-result.json`
- `smoke.targets` 是本地 readiness test 与 release smoke 的共享契约真源，包含端点、必需文件与统一响应安全头
- 本目录中的本地调试脚本不改变生产发布主线，只作为开发/排障辅助
