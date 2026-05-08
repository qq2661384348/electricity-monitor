# Electricity Monitor 后端项目

高性能电力监控系统后端 API，基于 Rust + Axum + Diesel。

## 文档导航

完整文档已整理到 `./docs/` 目录：

- **[ 文档索引](./docs/INDEX.md)** - 所有文档的导航入口
- **[ 快速开始](./docs/guides/QUICKSTART.md)** - 5分钟快速启动指南
- **[ 架构设计](./docs/architecture/ARCHITECTURE.md)** - 完整架构文档
- **[ API参考](./docs/api/API_REFERENCE.md)** - API接口文档
- **[ 项目详情](./docs/README.md)** - 详细项目说明

## 快速开始

Linux：

```bash
# 1. 安装系统依赖
sudo apt-get update
sudo apt-get install -y build-essential libpq-dev libssl-dev pkg-config postgresql-client redis-tools

# 2. 准备本地运行时配置
cp config/development.toml.example config/development.toml

# 3. 将 config/development.toml 中的 database.password 改成当前本地 PostgreSQL 的真实密码或非空开发值
#    同时填写 qq_bot.api_url、qq_bot.public_qq_number、qq_bot.bearer_token，以及 public_site.domain / public_site.port
#    如需启用邮箱登录和邮箱通知，再填写 email.smtp_password 或 APP__EMAIL__SMTP_PASSWORD(_FILE)

# 4. 配置环境
export APP_ENV=development

# 5. 确保 PostgreSQL 和 Redis 已启动，并可通过 127.0.0.1:5432 / 127.0.0.1:6379 访问
#    它们可以是系统服务，也可以是 Docker 映射到本地端口的容器。

# 6. 运行迁移（统一命令，不要求安装 diesel_cli）
cargo run --bin migrate

# 7. 启动服务
cargo run

# 8. 测试 API
curl http://localhost:8000/api/health
```

Windows 原生：

```powershell
Copy-Item config/development.toml.example config/development.toml
# 继续编辑 config/development.toml，把 database.password 改成当前本地 PostgreSQL 的真实密码或非空开发值，并填写 qq_bot.api_url、qq_bot.public_qq_number、qq_bot.bearer_token 和 public_site.domain / public_site.port；如需启用邮箱登录和邮箱通知，再填写 email.smtp_password 或 APP__EMAIL__SMTP_PASSWORD(_FILE)
$env:APP_ENV="development"
cargo run --bin migrate
cargo run
curl http://localhost:8000/api/health
```

也可以直接运行统一后端自检脚本：

```bash
bash scripts/backend-checks.sh
```

```powershell
powershell -ExecutionPolicy Bypass -File scripts/backend-checks.ps1
```

## 技术栈概览

**Web框架**: Axum 0.8 + Tokio + Tower  
**数据库**: Diesel 2.3 + diesel-async 0.7 + PostgreSQL
**性能优化**: sonic-rs (SIMD加速JSON) + mimalloc (高性能内存分配器)  
**认证**: JWT + bcrypt  
**配置**: TOML分层配置  

## 性能特性

- JSON序列化速度提升 **2-3倍** (sonic-rs)
- 内存分配性能提升 **20-40%** (mimalloc)
- 内存占用优化 **15-20%** (Axum)
- 编译时类型检查 **零运行时错误** (Diesel)

## 项目结构

```
src/
├── bootstrap/          # 启动装配、日志、路由、运行时拆分入口
├── config/              # 配置管理
├── domain/             # 领域层（DDD）
├── handlers/           # HTTP处理器
├── infrastructure/     # 基础设施层
├── middleware/         # 中间件
├── modules/            # 渐进式模块化边界（当前含 auth 模块样板）
├── routes/             # 路由定义
├── errors.rs           # 统一错误处理
├── state.rs            # 应用状态
└── main.rs             # 程序入口
```

```text
deploy/
├── Dockerfile                # GitHub Actions / release 镜像构建文件
├── Dockerfile.dockerignore   # 与 deploy/Dockerfile 配套的构建忽略规则
├── build.sh                  # 本地 Docker 调试脚本
├── docker-compose.local.yml  # 本地 Docker 调试编排
├── compose.release.yml       # release 包内 compose 模板
├── deploy.sh                 # release 包内一键部署脚本模板
├── smoke.targets             # readiness / smoke 共用检查契约
├── release.env.example       # release 包内 .env 模板
└── README.release.md         # release 包内说明模板
```

## 环境配置

### 开发环境（Linux）

```bash
cp config/development.toml.example config/development.toml
# 继续编辑 config/development.toml，把 database.password 改成当前本地 PostgreSQL 的真实密码，并填写 qq_bot.api_url、qq_bot.public_qq_number、qq_bot.bearer_token 和 public_site.domain / public_site.port；如需启用邮箱登录和邮箱通知，再填写 email.smtp_password 或 APP__EMAIL__SMTP_PASSWORD(_FILE)
export APP_ENV=development
export RUST_LOG=debug
# development 环境只允许连接本地 PostgreSQL / Redis；
# 本地服务可以是系统服务，也可以是映射到 127.0.0.1 的 Docker 容器。
```

### 开发环境（Windows 原生）

```powershell
Copy-Item config/development.toml.example config/development.toml
# 继续编辑 config/development.toml，把 database.password 改成当前本地 PostgreSQL 的真实密码，并填写 qq_bot.api_url、qq_bot.public_qq_number、qq_bot.bearer_token 和 public_site.domain / public_site.port；如需启用邮箱登录和邮箱通知，再填写 email.smtp_password 或 APP__EMAIL__SMTP_PASSWORD(_FILE)
$env:APP_ENV="development"
$env:RUST_LOG="debug"
# development 环境只允许连接本地 PostgreSQL / Redis
```

### 生产环境（Linux）

```bash
cp config/production.toml.example config/production.toml
export APP_ENV=production
export RUST_LOG=info
```

`config/` 目录下只能保留一个运行时 `.toml` 文件，文件名必须与环境一致：开发环境使用 `config/development.toml`，生产/发布环境使用 `config/production.toml`。两个运行时文件都不纳入版本控制。

认证与浏览器访问的关键运行时约束：

- `cors.allowed_origins` 使用逗号分隔字符串维护前端 Origin 白名单；开发模板默认是 `http://localhost:5173`。
- `auth.refresh_cookie_secure` 与 `auth.refresh_cookie_same_site` 控制 refresh cookie；生产环境要求 `refresh_cookie_secure = true`。
- `qq_bot.api_url`、`qq_bot.public_qq_number` 与 `qq_bot.bearer_token` 必须由部署者手动填写或通过 secret file 注入；模板不内置本地 mock 地址或任何线上环境值，机器人 QQ 也不能从管理员 QQ 或 NapCat 登录信息自动推断。
- `public_site.domain` 与 `public_site.port` 是 QQ 通知消息中公开访问链接的唯一配置真源，模板默认留空，运行时必须填写真实域名和端口。
- `admin.default_qq_number` 在生产环境不能留空，也不能保留模板占位值；只有显式配置的真实管理员 QQ 才会授予 `admin`。
- `[captcha]` 和 `[verification]` 控制第三方图形验证码参数、一次性 captcha token 有效期以及登录验证码长度；验证码 Redis key 会按 `qq` / `email` 渠道隔离。
- `[email]` 填写完整 SMTP host、user 和授权码后启用邮箱登录与邮箱电量通知；邮箱登录本轮始终授予普通用户，不走管理员提升。
- 登录成功后，refresh token 只通过 HTTPOnly Cookie 下发；前端只接收和持有 access token。

## 开发工具

```bash
cargo fmt      # 代码格式化
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test auth_integration_test
cargo test --test release_readiness_test
bun audit --cwd frontend
cargo audit -q
powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1
```

生成新 migration 或手动刷新 Diesel schema 时再安装 Diesel CLI：

```bash
cargo install diesel_cli --no-default-features --features postgres
```

## 部署

生产部署已调整为 GitHub Actions 手动触发打包，仓库内部署资产统一收敛在 `deploy/`：

- PR / 手动质量门禁：`.github/workflows/ci.yml`
- 工作流：`.github/workflows/docker-build.yml`
- 镜像构建：`deploy/Dockerfile`
- artifact 部署契约：在 GitHub Actions 中触发 `.github/workflows/docker-build.yml`，下载 release artifact，上传到 `<server>` 的 `<release-root>/<git-tag>`，在服务器准备 `.env` 与 `secrets/` 后执行 release 包内 `deploy.sh` / `smoke.sh`
- 运行时配置：工作流会先将 `config/production.toml.example` 复制为 `config/production.toml` 再构建镜像
- 生产模板中的 `cors.allowed_origins`、`auth.refresh_cookie_secure`、`qq_bot.api_url`、`qq_bot.public_qq_number`、`public_site.domain`、`public_site.port` 和 `admin.default_qq_number` 必须在发布前补成真实生产值；`qq_bot.bearer_token` 和 `email.smtp_password` 生产环境必须通过 secret file 注入
- release 模板：`deploy/compose.release.yml`、`deploy/release.env.example`、`deploy/deploy.sh`
- GitHub Actions 会拆分产出 app release artifact 与 infra images artifact；日常发布只需要上传 app 包，首次部署或 PostgreSQL / Redis 镜像版本变更时再把 infra 包解压到同一 release 目录
- 服务器只执行 `docker load` 和 `docker compose`，不会从外部 registry 拉取镜像；`deploy.sh` 会在启动前校验 release manifest 中声明的归档 SHA256、加载后的镜像摘要，并检查 app、PostgreSQL 和 Redis 镜像是否已离线可用
- release compose 服务包含 `postgres`、`redis`、一次性 `migrate` 和 `app`，默认绑定 `127.0.0.1:11450`
- 日常发布默认不重建 PostgreSQL / Redis；基础服务镜像摘要变化时脚本会 fail-fast，只有显式设置 `DEPLOY_RECREATE_BASE_SERVICES=true` 才会重建基础服务容器
- release 应用容器默认设置 `MIMALLOC_PURGE_DELAY=0` 与 `MIMALLOC_PURGE_DECOMMITS=1`，配合流式后台批处理降低长跑服务的 RSS 高水位
- PostgreSQL / Redis 持久数据目录通过 release `.env` 中的 `POSTGRES_DATA_DIR` 与 `REDIS_DATA_DIR` 配置；生产部署建议使用 release 目录之外的稳定 `<data-root>`
- smoke 契约：`deploy/smoke.targets`，由 `tests/runtime/release_readiness_test.rs` 与 `deploy/smoke.sh` 共用，包含端点、必需文件与统一响应安全头
- release manifest：artifact 内的 `release/release-manifest.json`
- 本地 Docker 调试：`deploy/build.sh`、`deploy/docker-compose.local.yml`
- `static/` 由前端 `build:prod` 在本地或 CI 生成，仓库只保留目录占位，不再跟踪构建产物
- 前端行为测试：在 `frontend/` 目录执行 `bun run test`，由 `Vitest + Testing Library + MSW` 驱动并纳入 `.github/workflows/ci.yml`
- `cargo audit -q` 已纳入 `.github/workflows/ci.yml` 阻断门禁
1. 准备并推送发布 tag。
2. 在 GitHub Actions 中触发发布工作流并指定 `git_tag`，下载 release artifact 后上传到目标服务器。
3. 下载 `electricity-monitor-app-release-<git-tag>`；首次部署或基础镜像变更时，再下载并解压 `electricity-monitor-infra-images-<git-tag>` 到同一个 release 父目录。
4. 如果手动部署，在服务器解压后准备 `.env` 与 `secrets/` 中的 Compose secrets 文件，并把数据库、JWT、QQ token 和 SMTP 授权码 secret file 权限收紧到仅 owner 可读写。
5. 执行 release 包中的 `deploy.sh`，必要时再执行 `smoke.sh`；smoke 会继续校验运行时端点、必需文件与统一响应安全头。
6. 部署结果会写入 release 目录下的 `deploy-result.json`。

详见 **[Docker 部署指南](./docs/guides/DOCKER_DEPLOYMENT.md)** 和 **[deploy 目录说明](./deploy/README.md)**。

## 更多文档

访问 **[docs/INDEX.md](./docs/INDEX.md)** 查看完整文档索引。

## 许可证说明

本项目采用 MIT License。

---

**Electricity Monitor Team** - 2025
