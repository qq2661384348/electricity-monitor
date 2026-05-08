# 快速启动指南

## 当前项目形态

Electricity Monitor 是 Rust + Axum + Diesel 后端与 React + Vite 前端同仓项目。后端运行依赖 PostgreSQL 与 Redis；开发环境固定使用 `APP_ENV=development`，并只允许连接本地 PostgreSQL / Redis。本地服务可以是系统服务，也可以是映射到 `127.0.0.1` 的 Docker 容器。

## Linux 快速启动

### 1. 安装后端依赖

```bash
sudo apt-get update
sudo apt-get install -y build-essential libpq-dev libssl-dev pkg-config postgresql-client redis-tools
```

### 2. 准备本地 PostgreSQL 和 Redis

确保 PostgreSQL 和 Redis 可以通过local environment地址访问：

```bash
pg_isready -h 127.0.0.1 -p 5432 -U postgres
redis-cli -h 127.0.0.1 -p 6379 ping
```

PostgreSQL / Redis 可以由系统服务提供，也可以由 Docker 容器通过端口映射提供。若数据库还不存在，先创建：

```bash
createdb -h 127.0.0.1 -U postgres electricity_dev
```

如果 Docker PostgreSQL 容器启用了 trust 认证，应用配置里仍要把 `database.password` 改成一个非空开发值，因为项目会在配置阶段阻止占位密码继续运行。

### 3. 准备运行时配置

```bash
cp config/development.toml.example config/development.toml
# 继续编辑 config/development.toml，把 database.password 改成当前本地 PostgreSQL 的真实密码或非空开发值
# 同时填写 qq_bot.api_url、qq_bot.public_qq_number、qq_bot.bearer_token 和 public_site.domain / public_site.port
export APP_ENV=development
export RUST_LOG=debug
```

### 4. 运行迁移并启动后端

```bash
cargo run --bin migrate
cargo run
```

### 5. 验证 API

```bash
curl http://localhost:8000/api/health
curl http://localhost:8000/api/health/db
```

也可以运行 Linux 后端统一自检：

```bash
bash scripts/backend-checks.sh
```

## Windows 原生快速启动

Windows 原生开发需要 PostgreSQL、Redis 和 Rust 工具链。PostgreSQL 如果不在标准安装路径，请通过用户环境变量设置 `POSTGRES_HOME` 或 `PQ_LIB_DIR`，不要把个人机器上的绝对路径写回 `.cargo/config.toml`。只有生成新迁移或手动刷新 schema 时才需要 Diesel CLI。

```powershell
Copy-Item config/development.toml.example config/development.toml
# 继续编辑 config/development.toml，把 database.password 改成当前本地 PostgreSQL 的真实密码，并填写 qq_bot.api_url、qq_bot.public_qq_number、qq_bot.bearer_token 和 public_site.domain / public_site.port
$env:APP_ENV="development"
$env:RUST_LOG="debug"
cargo run --bin migrate
cargo run
curl http://localhost:8000/api/health
```

Windows 后端统一自检：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/backend-checks.ps1
```

## 数据库迁移

项目已存在完整迁移目录，日常只需要运行：

```bash
cargo run --bin migrate
```

新增迁移时再安装并使用 Diesel CLI：

```bash
cargo install diesel_cli --no-default-features --features postgres
diesel migration generate your_migration_name
```

## 🔧 开发指南

### 添加新的API端点

1. **定义领域模型** (`src/domain/models/your_model.rs`):
```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}
```

2. **创建处理器** (`src/handlers/user.rs`):
```rust
use axum::{Json, extract::State};
use crate::state::AppState;
use crate::domain::models::User;
use crate::errors::Result;

pub async fn get_user(
    State(state): State<AppState>
) -> Result<Json<User>> {
    // 实现逻辑
    todo!()
}
```

3. **注册路由** (`src/routes/api.rs`):
```rust
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/users/{id}", get(handlers::get_user))  // 新增
}
```

### 数据库操作示例

```rust
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

pub async fn create_user(
    pool: &DbPool,
    username: &str,
    email: &str,
) -> Result<User> {
    let mut conn = pool.get().await?;
    
    diesel::insert_into(users::table)
        .values((
            users::username.eq(username),
            users::email.eq(email),
        ))
        .get_result(&mut conn)
        .await
        .map_err(Into::into)
}
```

## 📝 配置环境变量

### 开发环境

Linux：

```bash
export APP_ENV=development
export RUST_LOG=debug,electricity_monitor_backend=trace
```

Windows 原生：

```powershell
$env:APP_ENV="development"
$env:RUST_LOG="debug,electricity_monitor_backend=trace"
```

### 生产环境

```bash
export APP_ENV=production
export RUST_LOG=info
export APP__DATABASE__PASSWORD_FILE="/run/secrets/app_database_password"
export APP__JWT__SECRET_FILE="/run/secrets/app_jwt_secret"
```

## 🐛 常见问题

### 问题1: 编译错误 "diesel schema not found"

**原因**: 未运行数据库迁移

**解决**:
```bash
cargo run --bin migrate
```

### 问题2: 连接数据库失败

**检查清单**:
- [ ] 数据库服务是否运行
- [ ] Redis 服务是否运行
- [ ] 配置文件中的连接信息是否正确
- [ ] 用户名密码是否正确

### 问题3: 编译时间过长

**原因**: 首次编译需要下载和编译所有依赖

**优化**:
```bash
# 使用更快的链接器（可选）
cargo install -f cargo-binutils
rustup component add llvm-tools-preview
```

## 🚢 生产部署

### 构建发布版本

```bash
cargo build --release
```

生产发布主线不是从开发机或服务器源码目录直接构建，而是通过 GitHub Actions 生成 release artifact，再在服务器侧执行 release 包中的部署脚本。开发机本地 release 构建只用于调试。

### 部署检查清单

- [ ] 将 `config/production.toml.example` 复制为 `config/production.toml`
- [ ] 确认 `config/production.toml` 中不保留开发环境数据库密码
- [ ] 设置环境变量 `APP__JWT__SECRET_FILE`
- [ ] 准备 `APP__DATABASE__PASSWORD_FILE`
- [ ] 设置 `RUST_LOG=info` 减少日志输出
- [ ] 配置反向代理（Nginx/Caddy）
- [ ] 配置 HTTPS 证书
- [ ] 使用 release 包内 `deploy.sh` 和 `compose.release.yml` 完成部署

## 📚 文档链接

- [README.md](../README.md) - 项目概述和使用说明
- [ARCHITECTURE.md](../architecture/ARCHITECTURE.md) - 详细架构设计文档
- [API_REFERENCE.md](../api/API_REFERENCE.md) - API接口文档
- [Axum文档](https://docs.rs/axum/latest/axum/)
- [Diesel文档](https://diesel.rs/)

## 🎯 性能基准

**预期性能指标**:
- 吞吐量: >100,000 req/s（单机）
- P99延迟: <10ms
- 内存占用: <100MB（空载）

**性能提升**:
- JSON序列化速度: 相比serde_json快**2-3倍**
- 内存占用: 相比Actix-web低**15-20%**
- 编译时类型安全: **100%**保证

## 💡 提示

- 使用 `cargo watch -x run` 实现热重载开发
- 使用 `cargo clippy` 检查代码质量
- 使用 `cargo test` 运行测试
- 查看 `ARCHITECTURE.md` 了解设计决策

---

**祝开发顺利！** 🎉

如有问题，请参考文档或检查代码注释。
