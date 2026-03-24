# Deploy Assets

`deploy/` 是仓库内部与部署相关文件的统一入口。

## Production release

- `Dockerfile`: GitHub Actions 构建 release 镜像时使用的 Dockerfile
- `Dockerfile.dockerignore`: 与 `Dockerfile` 配套的构建忽略规则
- `compose.release.yml`: release 包内的 `compose.yaml` 模板
- `deploy.sh`: release 包内的一键部署脚本模板
- `release.env.example`: release 包内的 `.env.example` 模板
- `README.release.md`: release 包内的 `README.md` 模板

## Local Docker debugging

- `build.sh`: 本地 Docker 调试入口
- `docker-compose.local.yml`: 本地 Docker 调试使用的 compose 文件

## Source of truth

- 生产发布主线：`.github/workflows/docker-build.yml`
- 服务器部署方式：下载 GitHub Actions artifact，解压后执行包内 `deploy.sh`
- 本目录中的本地调试脚本不改变生产发布主线，只作为开发/排障辅助
