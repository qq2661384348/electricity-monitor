# Docker 部署指南

本文档介绍如何使用 Docker Compose 部署电力监控系统。

## 快速开始

```bash
# 启动服务
./build.sh up

# 查看日志
./build.sh logs

# 停止服务
./build.sh down
```

## 运维脚本

项目提供 `build.sh` 脚本，封装了常用的 Docker 操作命令。

### 命令列表

| 命令 | 说明 | 示例 |
|------|------|------|
| `build [TAG]` | 构建镜像 | `./build.sh build v1.0.0` |
| `up` | 启动服务（含构建） | `./build.sh up` |
| `down` | 停止服务 | `./build.sh down` |
| `restart` | 重启服务 | `./build.sh restart` |
| `logs [SERVICE]` | 查看日志 | `./build.sh logs app` |
| `status` | 查看服务状态 | `./build.sh status` |
| `export [FILE]` | 导出镜像为 tar 文件 | `./build.sh export` |
| `clean` | 清理未使用镜像 | `./build.sh clean` |
| `help` | 显示帮助 | `./build.sh help` |

### 日志查看

```bash
# 查看所有服务日志
./build.sh logs

# 只看应用日志
./build.sh logs app

# 只看 Redis 日志
./build.sh logs redis
```

## 配置管理

### 配置文件结构

```
config/
├── default.toml        # 基础配置（所有环境共用）
├── development.toml    # 开发环境覆盖
└── production.toml     # 生产环境覆盖
```

### 配置加载优先级

1. `config/default.toml` — 最低优先级
2. `config/{APP_ENV}.toml` — 环境配置覆盖
3. 环境变量 `APP__XXX__YYY` — 最高优先级

### 镜像自包含

镜像内已打包所有必要文件：
- `config/*.toml` — 配置文件
- `static/` — 前端静态文件
- `migrations/` — 数据库迁移

**无需挂载任何目录**，镜像可独立运行。

### 环境变量覆盖

敏感配置通过环境变量覆盖（优先级最高）：

```yaml
environment:
  - APP_ENV=production
  - APP__DATABASE__HOST=your-db-host
  - APP__DATABASE__PASSWORD=your-password
  - APP__JWT__SECRET=your-jwt-secret
```

**命名规则**：`APP__<SECTION>__<KEY>`（双下划线分隔）

### 修改配置

```bash
# 方式1：编辑 docker-compose.yml 中的 environment
vim docker-compose.yml

# 方式2：修改 config/*.toml 后重新构建
vim config/production.toml
./build.sh up  # 重新构建并启动
```

## 服务架构

```
┌─────────────────────────────────────────────────┐
│              docker-compose                      │
│                                                  │
│  ┌───────────────────┐    ┌──────────────────┐  │
│  │       app         │◄──►│      redis       │  │
│  │   (Rust 后端)     │    │   (纯内存模式)   │  │
│  │   :8000 → :11450  │    │                  │  │
│  │                   │    │                  │  │
│  │  内置: config/    │    │                  │  │
│  │        static/    │    │                  │  │
│  └───────────────────┘    └──────────────────┘  │
│                                                  │
└─────────────────────────────────────────────────┘
           ▲
           │ 环境变量覆盖敏感配置
           │ APP__DATABASE__PASSWORD=xxx
```

### 端口映射

| 服务 | 容器端口 | 宿主机端口 |
|------|----------|------------|
| app | 8000 | 11450 |
| redis | 6379 | 不暴露 |

### 访问地址

- **API 地址**: `http://localhost:11450`
- **健康检查**: `http://localhost:11450/api/health`

## Redis 配置

容器内的 Redis 采用纯内存模式运行：

```bash
redis-server --save "" --appendonly no --maxmemory 128mb --maxmemory-policy allkeys-lru
```

**说明**：
- `--save ""` — 禁用 RDB 持久化
- `--appendonly no` — 禁用 AOF 持久化
- `--maxmemory 128mb` — 限制最大内存
- `--maxmemory-policy allkeys-lru` — 内存满时 LRU 淘汰

**适用场景**：项目中 Redis 仅用于短生命周期的临时数据（限流计数、验证码、缓存），无需持久化。

## 服务器部署

将镜像部署到其他 Linux 服务器的完整流程。

### 1. 构建并导出镜像（开发机）

```bash
# 构建镜像
./build.sh build

# 导出为 tar 文件
./build.sh export
# 生成: electricity-monitor.tar
```

### 2. 传输到服务器

```bash
# 上传镜像和部署脚本
scp electricity-monitor.tar deploy.sh user@server:/opt/electricity/
```

### 3. 运行部署（服务器）

```bash
cd /opt/electricity
chmod +x deploy.sh
./deploy.sh
```

`deploy.sh` 会自动完成：
- 加载镜像（检测到 tar 文件时）
- 清理旧容器
- 创建 Docker 网络
- 启动 Redis 和应用
- 健康检查

### 4. 管理服务

```bash
# 查看日志
docker logs -f electricity-app

# 停止服务
docker stop electricity-app electricity-redis

# 启动服务
docker start electricity-redis electricity-app

# 重启应用
docker restart electricity-app
```

### 5. 配置覆盖

编辑 `deploy.sh` 中的配置区或启动参数：

```bash
# 修改端口
APP_PORT="8080:8000"

# 添加环境变量覆盖
docker run ... \
    -e APP__DATABASE__HOST=prod-db \
    -e APP__LOGGING__LEVEL=warn \
    ...
```

## 常见问题

### 1. 如何查看容器资源使用？

```bash
./build.sh status
```

### 2. 如何完全重建镜像？

```bash
./build.sh down
./build.sh clean
./build.sh up
```

### 3. 如何修改日志级别？

**方式1：环境变量（推荐）**

```bash
# 编辑 deploy.sh 或 docker-compose.yml
-e APP__LOGGING__LEVEL=debug
```

**方式2：修改配置文件后重新构建**

```toml
# config/default.toml
[logging]
level = "debug"  # 可选: error, warn, info, debug, trace
```

### 4. 数据库连接配置在哪？

`config/production.toml` 中的 `[database]` 部分：

```toml
[database]
host = "your-db-host"
port = 5432
username = "postgres"
password = "your-password"
database = "electricity_pro"
```

## 相关文档

- [快速启动指南](./QUICKSTART.md)
- [数据库迁移指南](./DATABASE_MIGRATION.md)
- [API 参考](../api/API_REFERENCE.md)
