# Electricity Monitor Backend

高性能电力监控系统后端API - 基于 Rust + Axum + Diesel

## 📚 文档导航

完整文档已整理到 `./docs/` 目录：

- **[📖 文档索引](./docs/INDEX.md)** - 所有文档的导航入口
- **[📘 快速开始](./docs/guides/QUICKSTART.md)** - 5分钟快速启动指南
- **[🏗️ 架构设计](./docs/architecture/ARCHITECTURE.md)** - 完整架构文档
- **[🔌 API参考](./docs/api/API_REFERENCE.md)** - API接口文档
- **[📋 项目详情](./docs/README.md)** - 详细项目说明

## 🚀 快速开始

```powershell
# 1. 安装依赖
cargo install diesel_cli --no-default-features --features postgres

# 2. 配置环境
$env:APP_ENV="development"

# 3. 确保本地 PostgreSQL 和本地 Redis 已启动

# 4. 运行迁移（统一命令）
cargo run --bin migrate

# 5. 启动服务
cargo run

# 6. 测试API
curl http://localhost:8000/api/health
```

## ⚡ 技术栈

**Web框架**: Axum 0.8 + Tokio + Tower  
**数据库**: Diesel 2.2 + diesel-async + PostgreSQL  
**性能优化**: sonic-rs (SIMD加速JSON) + mimalloc (高性能内存分配器)  
**认证**: JWT + bcrypt  
**配置**: TOML分层配置  

## 📊 性能特点

- JSON序列化速度提升 **2-3倍** (sonic-rs)
- 内存分配性能提升 **20-40%** (mimalloc)
- 内存占用优化 **15-20%** (Axum)
- 编译时类型检查 **零运行时错误** (Diesel)

## 🏗️ 项目结构

```
src/
├── config/              # 配置管理
├── domain/             # 领域层（DDD）
├── handlers/           # HTTP处理器
├── infrastructure/     # 基础设施层
├── middleware/         # 中间件
├── routes/             # 路由定义
├── errors.rs           # 统一错误处理
├── state.rs            # 应用状态
└── main.rs             # 程序入口
```

## 🌍 环境配置

### 开发环境 (Windows)
```powershell
$env:APP_ENV="development"
$env:RUST_LOG="debug"
# development 环境只允许连接本地 PostgreSQL / Redis
```

### 生产环境 (Linux)
```bash
export APP_ENV=production
export RUST_LOG=info
```

## 🔧 开发工具

```bash
cargo fmt      # 代码格式化
cargo clippy   # 代码检查
cargo test     # 运行测试
```

## 📦 部署

生产部署已调整为 GitHub Actions 手动触发打包：

1. 在 GitHub Actions 中手动触发发布工作流并指定 `git_tag`
2. 下载生成的 release artifact
3. 在服务器解压后执行 `deploy.sh`

详见 **[Docker 部署指南](./docs/guides/DOCKER_DEPLOYMENT.md)**。

## 📖 更多文档

访问 **[docs/INDEX.md](./docs/INDEX.md)** 查看完整文档索引

## 📝 许可证

MIT License

---

**Electricity Monitor Team** - 2025
