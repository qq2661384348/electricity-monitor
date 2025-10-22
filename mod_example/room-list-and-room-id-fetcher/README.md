# 房间信息爬取工具 (Room Fetcher)

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

高性能、高并发的房间信息爬取工具，使用 Rust 编写，相比 Python 版本性能提升 **100+ 倍**。

## ✨ 特性

- 🚀 **高性能**：异步并发架构，吞吐量达 **1160+ 房间/秒**
- ⚡ **高并发**：支持最高 50 并发请求，智能限流控制
- 🛡️ **高可靠**：自动重试机制（指数退避），优雅错误处理
- 📦 **零依赖运行**：编译后单一可执行文件，无需安装 Rust 环境
- 📊 **结构化日志**：DEBUG 级别日志，便于调试和监控
- 💾 **JSON 输出**：格式化的 JSON 文件，便于后续处理

## 📊 性能对比

| 指标 | Python 版本 | Rust 版本 | 提升 |
|------|------------|-----------|------|
| **总耗时** | ~5-10 分钟 | **4.98 秒** | **60-120倍** |
| **吞吐量** | ~10-20 房间/秒 | **1160+ 房间/秒** | **58-116倍** |
| **并发支持** | 无 | 50 并发 | **∞** |
| **内存占用** | ~50-100MB | ~10-20MB | **2.5-10倍** |
| **错误恢复** | 无 | 自动重试 | ✅ |

## 🏗️ 技术架构

### 核心技术栈

- **reqwest 0.12**：高性能 HTTP 客户端，内置连接池
- **tokio 1.x**：Rust 异步运行时，事实标准
- **serde/serde_json 1.0**：零拷贝 JSON 序列化
- **anyhow 1.0**：现代错误处理
- **tracing 0.1**：结构化日志系统

### 架构设计

```
┌─────────────────────────────────────────────────────┐
│                    Main Entry                       │
│  - 初始化日志系统                                    │
│  - 创建 HTTP 客户端和爬取器                          │
│  - 执行爬取并输出结果                                │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│                 RoomFetcher                         │
│  - 管理4层级联爬取流程                               │
│  - 并发控制（Semaphore 限流）                        │
│  - 错误处理与优雅降级                                │
└──────────────────┬──────────────────────────────────┘
                   │
     ┌─────────────┴─────────────┐
     ▼                           ▼
┌─────────────┐          ┌──────────────┐
│ RoomClient  │          │  数据模型     │
│ - HTTP请求  │          │  - API响应   │
│ - 重试机制  │          │  - 房间信息  │
│ - 连接池    │          │             │
└─────────────┘          └──────────────┘
```

### 并发模型

```
Level 1 (串行)
    ↓
    校区列表
    ↓
Level 2-4 (并发，Semaphore 限流 50)
    ↓
    ┌────────┬────────┬────────┐
    │ 建筑1   │ 建筑2   │ ...    │
    │ (异步)  │ (异步)  │ (异步) │
    └────────┴────────┴────────┘
         ↓        ↓        ↓
    楼层 → 房间（并发获取）
```

## 📦 安装与使用

### 前置要求

- **Rust 1.70+**（仅编译时需要）
- **Windows/Linux/macOS**（跨平台支持）

### 编译

```bash
# 克隆或进入项目目录
cd pach

# 编译 Release 版本（优化性能）
cargo build --release

# 可执行文件位于 target/release/room-fetcher.exe (Windows)
# 或 target/release/room-fetcher (Linux/macOS)
```

### 运行

```bash
# Windows
.\target\release\room-fetcher.exe

# Linux/macOS
./target/release/room-fetcher
```

### 输出

程序运行完成后，会在 `output/` 目录生成 `rooms.json` 文件：

```json
[
  {
    "roompath": "箭盘校区/箭盘校区学生公寓/1层/101",
    "roomid": "1325"
  },
  {
    "roompath": "箭盘校区/箭盘校区学生公寓/1层/102",
    "roomid": "1326"
  }
]
```

## 🔧 配置说明

### 并发控制

在 `main.rs` 中修改并发数（默认 50）：

```rust
let fetcher = RoomFetcher::new(client, 50); // 修改这里
```

### 日志级别

通过环境变量控制日志级别：

```bash
# Windows PowerShell
$env:RUST_LOG="room_fetcher=debug"
.\target\release\room-fetcher.exe

# Linux/macOS
RUST_LOG=room_fetcher=info ./target/release/room-fetcher
```

可选级别：`error`, `warn`, `info`, `debug`, `trace`

### 重试策略

在 `client.rs` 中修改重试配置：

```rust
const MAX_RETRIES: u32 = 3; // 最大重试次数
let delay = Duration::from_millis(100 * 2u64.pow(attempt - 1)); // 指数退避
```

## 📁 项目结构

```
pach/
├── Cargo.toml              # 依赖配置
├── README.md               # 本文档
├── .gitignore              # Git 忽略规则
├── src/
│   ├── main.rs             # 程序入口
│   ├── models.rs           # 数据模型定义
│   ├── parser.rs           # JSON 解析器（处理 BOM + 双重编码）
│   ├── client.rs           # HTTP 客户端封装
│   └── fetcher.rs          # 核心爬取逻辑
├── output/
│   └── rooms.json          # 输出文件（自动生成）
└── target/
    └── release/
        └── room-fetcher    # 编译产物
```

## 🛠️ 开发指南

### 代码检查

```bash
# 运行 Clippy（代码检查）
cargo clippy --release -- -D warnings

# 格式化代码
cargo fmt

# 运行测试
cargo test
```

### 添加新功能

1. **修改数据模型**：编辑 `src/models.rs`
2. **调整爬取逻辑**：编辑 `src/fetcher.rs`
3. **自定义输出格式**：编辑 `src/main.rs` 中的 `save_to_json`

### 性能优化建议

- **调整并发数**：根据网络带宽和目标服务器性能调整
- **连接池大小**：在 `client.rs` 中修改 `pool_max_idle_per_host`
- **超时配置**：调整 `connect_timeout` 和 `timeout` 参数

## ⚠️ 注意事项

1. **合理并发**：避免过高并发导致目标服务器限流
2. **网络依赖**：需要稳定的网络连接
3. **数据时效性**：爬取的数据以运行时为准
4. **使用规范**：遵守目标网站的 robots.txt 和服务条款

## 🐛 故障排查

### 问题：编译失败

**解决方案**：
- 确保 Rust 版本 ≥ 1.70：`rustc --version`
- 更新依赖：`cargo update`
- 清理缓存：`cargo clean && cargo build --release`

### 问题：运行时超时

**解决方案**：
- 检查网络连接
- 降低并发数（如改为 10-20）
- 增加超时时间（在 `client.rs` 中修改）

### 问题：部分数据缺失

**解决方案**：
- 查看日志中的 WARN/ERROR 信息
- 检查目标 API 是否有变化
- 确认网络稳定性

## 📝 更新日志

### v0.1.0 (2025-10-18)

- ✅ 初始版本发布
- ✅ 实现4层级联爬取
- ✅ 支持高并发（50并发）
- ✅ 自动重试机制
- ✅ JSON 输出支持
- ✅ 完整日志系统

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 🙏 致谢

- 原始 Python 版本项目（位于 `_old/` 目录）
- Rust 社区提供的优秀生态工具
- tokio、reqwest、serde 等开源项目

## 📧 联系方式

如有问题或建议，欢迎提交 Issue 或 Pull Request。

---

**Made with ❤️ and Rust**
