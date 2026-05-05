# 架构设计文档

## 项目概述

**项目名称**: Electricity Monitor 后端  
**技术栈**: Rust + Axum + Diesel-async + PostgreSQL  
**架构模式**: DDD (领域驱动设计) + 分层架构  
**性能优化**: SIMD加速(sonic-rs) + LTO + 异步连接池

## 技术选型决策

### Web 框架：Axum 0.8

**选择理由**:
- 性能接近Actix-web（差距<10%）
- 内存占用比Actix低15-20%
- Tokio官方框架，生态最强
- 基于Tower中间件系统，扩展性强

**性能数据** (TechEmpower 2025):
- 吞吐量: ⭐⭐⭐⭐☆ (接近最高)
- 延迟: ⭐⭐⭐⭐⭐ (最低)
- 内存: ⭐⭐⭐⭐⭐ (最优)

### ORM：Diesel 2.2 + diesel-async 0.5

**选择理由**:
- 编译时类型检查，零运行时错误
- crates.io 生产验证
- 支持PostgreSQL + MySQL双数据库
- diesel-async提供原生异步支持

**替代方案对比**:
- vs SQLx: Diesel类型安全更强，SQLx编译时SQL验证更灵活
- vs SeaORM: Diesel性能更优，文档更成熟

### JSON 序列化：sonic-rs 0.3

**选择理由**:
- 使用SIMD加速，性能是serde_json的2-3倍
- 与serde完全兼容，零迁移成本
- x86_64/aarch64优化

**性能基准**:
- twitter.json 反序列化: 快3倍
- canada.json 反序列化: 快2.3倍

### 连接池：deadpool

**选择理由**:
- diesel-async官方推荐
- 生产环境验证充分
- 性能与稳定性平衡

**替代方案**:
- bb8: 性能相当但有已知死锁问题
- qp: 性能最高（快2-3倍）但生态不够成熟

## 架构分层

```
┌─────────────────────────────────────────┐
│         Presentation Layer              │
│  (handlers/ + routes/ + middleware/)    │
│  - HTTP请求处理                          │
│  - 路由定义                              │
│  - JWT认证中间件                         │
│  - 日志跟踪中间件                        │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Application Layer               │
│          (domain/services/)             │
│  - 应用服务协调                          │
│  - 业务流程编排                          │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│           Domain Layer                  │
│          (domain/models/)               │
│  - 核心业务逻辑                          │
│  - 领域模型                              │
│  - 业务规则验证                          │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│      Infrastructure Layer               │
│  (infrastructure/database/repositories/)│
│  - 数据库访问                            │
│  - 外部服务集成                          │
│  - 技术细节实现                          │
└─────────────────────────────────────────┘
```

## 核心模块设计

### 1. 配置系统 (config/)

**设计原则**: 本地运行时配置 + 环境模板 + 环境变量覆盖

**配置优先级**:
1. `config/development.toml` 或 `config/production.toml` - 运行时配置，`config/` 下只能保留其一
2. 环境变量 `APP__*` - 最高优先级

运行前需要先复制模板：
- 开发环境：`config/development.toml.example -> config/development.toml`
- 生产/发布环境：`config/production.toml.example -> config/production.toml`

**实现亮点**:
- 使用 `config` crate 实现配置合并
- `OnceLock` 实现全局单例
- 类型安全的配置结构
- 开发环境数据库密码需要直接写入复制出来的 `config/development.toml`

### 2. 数据库层 (infrastructure/database/)

**连接池配置**:
- 最大连接数: 开发5, 生产20
- 最小空闲连接: 开发1, 生产5
- 连接超时: 30秒

**多数据库支持**:
```rust
pub enum DatabaseType {
    Postgres,  // 当前使用
    Mysql,     // 预留支持
}
```

**异步操作**:
```rust
pub type DbPool = AsyncPool<AsyncPgConnection>;
```

### 3. 错误处理 (errors.rs)

**统一错误类型**:
```rust
pub enum AppError {
    Database(diesel::result::Error),
    Config(String),
    Unauthorized(String),
    NotFound,
    Internal(String),
}
```

**HTTP响应转换**:
- 自动实现 `IntoResponse`
- 结构化JSON错误响应
- 敏感信息过滤

### 4. 中间件系统 (middleware/)

**JWT认证流程**:
1. 提取 `Authorization` 头
2. 验证 Bearer token 格式
3. 解码并验证JWT
4. 将 Claims 注入请求扩展

**日志跟踪**:
- 使用 `tower-http::trace::TraceLayer`
- 记录请求/响应时间
- 结构化日志输出

## 性能优化策略

### 编译时优化

**.cargo/config.toml**:
```toml
[build]
rustflags = ["-C", "target-cpu=native"]  # SIMD优化
```

**Cargo.toml**:
```toml
[profile.release]
opt-level = 3      # 最高优化级别
lto = "fat"        # 链接时优化
codegen-units = 1  # 单个代码生成单元
strip = true       # 去除符号信息
```

### 运行时优化

**内存分配器 - mimalloc**:
```rust
// src/main.rs
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

**优势**：
- 减少内存碎片，提升长时间运行的稳定性
- 优化多线程内存分配性能
- 相比系统默认分配器性能提升 20-40%

**Tokio配置**:
```rust
#[tokio::main(flavor = "multi_thread")]
```

**连接池调优**:
- 根据CPU核心数配置连接数
- 合理设置空闲连接数
- 连接超时保护

## 环境分离策略

### 开发环境 (Windows / Linux)

**特点**:
- 监听 `127.0.0.1:8000`
- 详细调试日志 (`level = "debug"`)
- 较小的连接池 (5个连接)
- 数据库: `electricity_dev`
- PostgreSQL / Redis 必须是本地依赖；可以是系统服务，也可以是映射到 `127.0.0.1` 的 Docker 容器。

### 生产环境 (Linux)

**特点**:
- 监听 `0.0.0.0:8000`
- 精简日志 (`level = "info"`)
- 大连接池 (20个连接)
- 数据库: `electricity_pro`

## 安全设计

### JWT认证

**Token结构**:
```rust
struct Claims {
    sub: String,        // QQ号
    user_id: String,    // 用户ID
    role: String,       // admin / user
    exp: usize,         // 过期时间
    iat: usize,         // 签发时间
}
```

**密钥管理**:
- 开发环境: 配置文件
- 生产环境: Compose secrets + `*_FILE` 覆盖

### 密码安全

- 使用 bcrypt 哈希
- 配置合理的计算成本
- 永不明文存储

## 扩展性设计

### 多数据库支持

**当前**: PostgreSQL  
**预留**: MySQL

**切换方式**:
```toml
[database]
type = "mysql"  # 修改此处即可
```

### 水平扩展

**无状态设计**:
- JWT无需session存储
- 应用层无状态
- 可通过负载均衡扩展

**数据库扩展**:
- 读写分离（预留）
- 分库分表（预留）

## 监控与可观测性

### 日志系统

**格式**: 结构化JSON（生产） / Pretty（开发）  
**级别**: DEBUG -> INFO -> WARN -> ERROR

**关键日志点**:
- 启动/关闭
- 配置加载
- 数据库连接
- 请求/响应
- 错误异常

### 健康检查

**端点**:
- `GET /api/health` - 基础检查
- `GET /api/health/db` - 数据库连接检查

## 部署建议

### 构建优化

```bash
# 启用性能优化编译
cargo build --release

# 二进制大小: ~10-15MB (stripped)
```

### 环境变量

**必需**:
- `APP_ENV`: development / production
- `APP_DATABASE_PASSWORD_SECRET_FILE`: 生产环境数据库密码 secret file 路径

**可选**:
- `APP_JWT_SECRET_SECRET_FILE`: JWT secret file 路径
- `RUST_LOG`: 日志级别

### 容器化（可选）

- 多阶段构建减小镜像
- 使用 `debian:bookworm-slim` 基础镜像
- 复制配置文件到容器

## 性能基准

**预期性能**:
- 吞吐量: >100K req/s (单机)
- P99延迟: <10ms
- 内存占用: <100MB (空载)

**对比标准方案提升**:
- JSON序列化: +200~300%
- 内存占用: -15~20%
- 类型安全: 100%编译时保证

## 维护指南

### 添加新API

1. `domain/models/` - 定义领域模型
2. `handlers/` - 实现HTTP处理器
3. `routes/` - 注册路由
4. `infrastructure/repositories/` - 数据访问（如需）

### 数据库迁移

```bash
# 创建迁移
diesel migration generate migration_name

# 运行迁移
diesel migration run

# 回滚迁移
diesel migration revert
```

### 配置变更

1. 修改 `config/*.toml`
2. 或使用环境变量覆盖
3. 重启应用生效

## 技术债务与改进计划

**已知限制**:
- [ ] 数据库schema需手动运行迁移生成
- [ ] JWT刷新token机制待实现
- [ ] 缺少API版本控制
- [ ] 缺少限流中间件

**未来优化**:
- [ ] 引入OpenAPI文档生成
- [ ] 集成Prometheus指标
- [ ] 实现分布式追踪
- [ ] GraphQL支持（可选）

---

**文档版本**: 1.0  
**最后更新**: 2025-10-21  
**维护团队**: Electricity Monitor Team
