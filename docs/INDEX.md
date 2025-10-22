# 📚 文档索引

欢迎使用 Electricity Monitor Backend 文档！

## 📖 文档分类

### 🏠 项目概述
- **[README.md](./README.md)** - 项目介绍、技术栈、环境配置、API端点

### 🏗️ 架构设计 (`./architecture/`)
- **[ARCHITECTURE.md](./architecture/ARCHITECTURE.md)** - 完整架构设计文档
  - 技术选型决策
  - 分层架构设计
  - 性能优化策略
  - 安全设计
  - 扩展性设计
  - 部署建议

### 📘 开发指南 (`./guides/`)
- **[QUICKSTART.md](./guides/QUICKSTART.md)** - 快速启动指南
  - 环境配置
  - 数据库初始化
  - 开发服务器启动
  - 常见问题解决
- **[BUILD_CONFIGURATION.md](./guides/BUILD_CONFIGURATION.md)** - 构建配置指南
  - PostgreSQL 自动检测配置
  - 编译优化设置
  - 链接问题解决方案
  - 常见构建错误排查

### 🔌 API 文档 (`./api/`)
- **[API_REFERENCE.md](./api/API_REFERENCE.md)** - API接口文档（待创建）
  - 健康检查接口
  - 认证接口
  - 业务接口

## 🚀 快速导航

### 新手入门
1. 先阅读 [README.md](./README.md) 了解项目概况
2. 按照 [QUICKSTART.md](./guides/QUICKSTART.md) 配置开发环境
3. 参考 [ARCHITECTURE.md](./architecture/ARCHITECTURE.md) 理解项目设计

### 开发参考
- 技术选型理由 → [ARCHITECTURE.md § 技术选型决策](./architecture/ARCHITECTURE.md#技术选型决策)
- 添加新API → [QUICKSTART.md § 开发指南](./guides/QUICKSTART.md#开发指南)
- 配置管理 → [ARCHITECTURE.md § 配置系统](./architecture/ARCHITECTURE.md#核心模块设计)
- 性能优化 → [ARCHITECTURE.md § 性能优化策略](./architecture/ARCHITECTURE.md#性能优化策略)

### 部署运维
- 环境配置 → [README.md § 环境配置](./README.md#环境配置)
- 生产部署 → [ARCHITECTURE.md § 部署建议](./architecture/ARCHITECTURE.md#部署建议)
- 性能基准 → [ARCHITECTURE.md § 性能基准](./architecture/ARCHITECTURE.md#性能基准)

## 📁 文档结构

```
docs/
├── INDEX.md                    # 本文件 - 文档索引
├── README.md                   # 项目主文档
├── architecture/               # 架构设计文档
│   └── ARCHITECTURE.md        # 详细架构设计
├── guides/                     # 开发指南
│   ├── QUICKSTART.md          # 快速启动
│   ├── BUILD_CONFIGURATION.md # 构建配置指南
│   ├── DEVELOPMENT.md         # 开发规范（待创建）
│   └── DEPLOYMENT.md          # 部署指南（待创建）
└── api/                        # API文档
    └── API_REFERENCE.md       # API参考
```

## 🔍 按主题查找

### 技术栈相关
- Axum框架使用 → [ARCHITECTURE.md § Web框架](./architecture/ARCHITECTURE.md)
- Diesel ORM → [ARCHITECTURE.md § 数据库层](./architecture/ARCHITECTURE.md)
- JWT认证 → [ARCHITECTURE.md § 中间件系统](./architecture/ARCHITECTURE.md)

### 配置相关
- TOML配置文件 → [README.md § 配置说明](./README.md#配置说明)
- 环境变量 → [QUICKSTART.md § 配置环境变量](./guides/QUICKSTART.md#配置环境变量)
- 数据库配置 → [QUICKSTART.md § 配置数据库](./guides/QUICKSTART.md#配置数据库)

### 开发相关
- 添加新端点 → [QUICKSTART.md § 添加新的API端点](./guides/QUICKSTART.md)
- 数据库操作 → [QUICKSTART.md § 数据库操作示例](./guides/QUICKSTART.md)
- 错误处理 → [ARCHITECTURE.md § 错误处理](./architecture/ARCHITECTURE.md)

### 性能相关
- 编译优化 → [ARCHITECTURE.md § 编译时优化](./architecture/ARCHITECTURE.md)
- SIMD加速 → [ARCHITECTURE.md § 性能优化](./architecture/ARCHITECTURE.md)
- 连接池调优 → [ARCHITECTURE.md § 运行时优化](./architecture/ARCHITECTURE.md)

## 📝 文档维护

- **最后更新**: 2025-10-21
- **文档版本**: 1.0
- **维护团队**: Electricity Monitor Team

## 💡 贡献指南

如需添加或更新文档：
1. 在对应分类目录下创建或修改文档
2. 更新本索引文件
3. 遵循 Markdown 格式规范
4. 包含必要的代码示例和图表

---

**提示**: 使用 Ctrl+F 在本页面搜索关键词快速定位文档。
