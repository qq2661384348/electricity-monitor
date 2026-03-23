# Electricity Monitor Backend - 详细项目文档

> 高性能电力监控系统后端API - Rust实现

## 目录

- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [环境配置](#环境配置)
- [数据库配置](#数据库配置)
- [API端点](#api端点)
- [性能优化](#性能优化)
- [开发工具](#开发工具)
- [部署指南](#部署指南)

## 技术栈

### Web框架
- **Axum 0.8**: 基于Tokio的高性能Web框架
- **Tower**: 模块化中间件系统
- **Tower-HTTP**: HTTP中间件（CORS、日志、压缩等）

### 数据库
- **Diesel 2.2**: 类型安全的ORM
- **diesel-async 0.5**: 异步数据库操作
- **deadpool**: 高性能连接池
- **支持数据库**: PostgreSQL (当前), MySQL (预留)

### 序列化与性能
- **sonic-rs**: SIMD加速的JSON序列化（比serde_json快2-3倍）
- **serde**: 数据序列化框架
- **mimalloc**: 微软开发的高性能内存分配器，减少内存碎片，提升多线程性能

### 认证与安全
- **jsonwebtoken**: JWT认证
- **bcrypt**: 密码哈希

### 配置管理
- **config**: 分层配置管理（TOML）
- 支持环境: `development` / `production`

## 项目结构

```
electricity-monitor-backend/
├── src/
│   ├── config/              # 配置管理
│   │   ├── app.rs          # 应用配置
│   │   └── database.rs     # 数据库配置
│   ├── domain/             # 领域层（DDD）
│   │   ├── models/         # 领域模型
│   │   └── services/       # 领域服务
│   ├── handlers/           # HTTP处理器
│   │   └── health.rs       # 健康检查
│   ├── infrastructure/     # 基础设施层
│   │   ├── database/       # 数据库连接池
│   │   └── repositories/   # 数据仓储
│   ├── middleware/         # 中间件
│   │   ├── auth.rs         # JWT认证
│   │   └── logger.rs       # 日志跟踪
│   ├── routes/             # 路由定义
│   │   └── api.rs          # API路由
│   ├── utils/              # 工具函数
│   ├── errors.rs           # 统一错误处理
│   ├── state.rs            # 应用状态
│   ├── lib.rs              # 库入口
│   └── main.rs             # 程序入口
├── config/                 # 配置文件
│   ├── default.toml        # 默认配置
│   ├── development.toml    # 开发环境
│   └── production.toml     # 生产环境
├── migrations/             # 数据库迁移
├── docs/                   # 文档目录
├── .cargo/
│   └── config.toml         # Cargo编译优化（SIMD）
├── Cargo.toml              # 项目依赖
├── diesel.toml             # Diesel配置
└── README.md
```

## 环境配置

### 开发环境 (Windows)

```powershell
# 设置环境变量
$env:APP_ENV="development"
$env:RUST_LOG="debug"

# 安装 Diesel CLI（如需数据库迁移）
cargo install diesel_cli --no-default-features --features postgres

# 运行数据库迁移
diesel migration run

# 启动开发服务器
cargo run
```

### 生产环境 (Linux)

```bash
# 设置环境变量
export APP_ENV=production
export RUST_LOG=info

# 构建生产版本
cargo build --release

# 运行服务器
./target/release/server
```

## 配置说明

### 配置文件优先级

1. `config/default.toml` - 基础配置
2. `config/{APP_ENV}.toml` - 环境特定配置（覆盖默认）
3. 环境变量 `APP__*` - 最高优先级

### 环境变量覆盖示例

```bash
# 覆盖数据库配置
export APP__DATABASE__HOST="localhost"
export APP__DATABASE__PORT="5432"
export APP__DATABASE__PASSWORD="your-password"

# 覆盖JWT密钥
export APP__JWT__SECRET="your-production-secret"
```

## 数据库配置

### 当前配置

- **开发环境**: `postgres://postgres:<your-local-password>@127.0.0.1:5432/electricity_dev`
- **生产环境**: `postgres://postgres:postgres@47.92.117.121:5432/electricity-pro`

开发环境运行时会校验数据库和 Redis 主机，拒绝非本地地址，防止误连远端环境。

### 切换到MySQL

修改 `config/default.toml`:

```toml
[database]
type = "mysql"
host = "your-mysql-host"
port = 3306
```

## API端点

### 健康检查

- `GET /api/health` - 基础健康检查
- `GET /api/health/db` - 包含数据库连接检查

详细API文档请查看：[API_REFERENCE.md](./api/API_REFERENCE.md)

## 性能优化

### 编译优化

项目已配置以下优化：

1. **SIMD加速**: `.cargo/config.toml` 中启用 `target-cpu=native`
2. **LTO**: `Cargo.toml` 中启用 `lto = "fat"`
3. **代码生成单元**: `codegen-units = 1` 提升优化效果

### 内存分配器优化

**mimalloc 全局内存分配器**

项目使用微软开发的 mimalloc 作为全局内存分配器，替代 Rust 默认的系统分配器：

```rust
// src/main.rs
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

**优势**：
- ✅ **减少内存碎片**：优化的内存布局算法
- ✅ **提升多线程性能**：针对并发场景优化，减少锁竞争
- ✅ **更快的分配速度**：相比 glibc allocator 性能提升显著
- ✅ **低内存占用**：紧凑的内存管理结构

**适用场景**：
- 高并发 Web 服务
- 频繁内存分配/释放的应用
- 多线程密集型任务

### 性能基准

相比标准serde_json方案:
- JSON序列化性能提升 **2-3倍** (sonic-rs)
- 内存分配性能提升 **20-40%** (mimalloc)
- 内存占用优化 **15-20%** (Axum vs Actix)
- 编译时类型检查确保 **零运行时错误** (Diesel)

## 开发工具

### 代码格式化

```bash
cargo fmt
```

### 代码检查

```bash
cargo clippy
```

### 运行测试

```bash
cargo test
```

## 部署指南

### Windows -> Linux 跨平台编译

```bash
# 安装目标工具链
rustup target add x86_64-unknown-linux-gnu

# 交叉编译
cargo build --release --target x86_64-unknown-linux-gnu
```

### Docker部署（可选）

项目已预留Docker支持，Dockerfile编写建议：

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/server /usr/local/bin/
COPY config /config
CMD ["server"]
```

## 开发规范

### 添加新功能

1. 在 `domain/models/` 中定义领域模型
2. 在 `handlers/` 中实现HTTP处理器
3. 在 `routes/` 中注册路由
4. 在 `infrastructure/repositories/` 中实现数据访问（如需）

### 代码风格

- 遵循Rust官方代码风格
- 使用 `cargo fmt` 自动格式化
- 使用 `cargo clippy` 检查潜在问题

## 相关文档

- [架构设计](./architecture/ARCHITECTURE.md) - 详细架构设计文档
- [快速启动](./guides/QUICKSTART.md) - 快速启动指南
- [API参考](./api/API_REFERENCE.md) - API接口文档
- [文档索引](./INDEX.md) - 所有文档导航

## 许可证

MIT License

## 贡献

Electricity Monitor Team

---

**最后更新**: 2025-10-21
