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
- `build.sh` 会在缺少 `config/development.toml` 时自动从 `config/development.toml.example` 复制一份本地运行时配置

## 当前真源

- 生产发布主线：`.github/workflows/docker-build.yml`
- PR / 手动质量门禁：`.github/workflows/ci.yml`
- 服务器部署方式：下载 GitHub Actions artifact，解压后执行包内 `deploy.sh`
- 本地 Docker 调试前会优先使用 `config/development.toml.example -> config/development.toml` 的本地运行时配置
- 生产敏感配置通过 Compose secrets 提供，并由 `.env` 中的 `*_SECRET_FILE` 指向宿主机文件
- 服务器部署时会读取 `release-manifest.json`，并在 release 目录写出 `deploy-result.json`
- `smoke.targets` 是本地 readiness test 与 release smoke 的共享契约真源
- 本目录中的本地调试脚本不改变生产发布主线，只作为开发/排障辅助
