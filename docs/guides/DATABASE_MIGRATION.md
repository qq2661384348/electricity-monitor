# 数据库迁移指南

本项目使用 Diesel 内嵌迁移执行数据库变更，配置完全基于 TOML 配置文件，无需 `.env` 文件。Diesel CLI 只在生成新迁移或手动刷新 schema 时需要，运行迁移不依赖目标环境安装 `diesel` 命令。

## 统一迁移命令

项目提供了统一的迁移工具 `migrate`，自动从 TOML 配置读取数据库连接信息。

### 基本用法

```bash
# 运行迁移（默认使用development环境）
cargo run --bin migrate

# 指定环境运行迁移
cargo run --bin migrate -- production

# 回滚最后一次迁移
cargo run --bin migrate -- --revert

# 在生产环境回滚
cargo run --bin migrate -- production --revert
```

### 工作原理

`migrate` 工具会：
1. 读取当前环境对应的运行时配置文件（`config/development.toml` 或 `config/production.toml`）中的数据库配置
2. 按当前环境校验数据库访问边界（例如 development 只允许本地数据库）
3. 构建数据库连接串
4. 通过编译进 `migrate` 二进制的内嵌 migrations 执行 `run_pending_migrations` 或 `revert_last_migration`

## 环境准备

### 1. 可选：安装 Diesel CLI

```bash
# 仅安装 PostgreSQL 支持
cargo install diesel_cli --no-default-features --features postgres

# 如需 MySQL 支持
cargo install diesel_cli --no-default-features --features postgres,mysql
```

只有生成新迁移、手动执行 `diesel migration list` 或刷新 `schema.rs` 时才需要 Diesel CLI；`cargo run --bin migrate` 和 release 包内的 `/app/migrate` 不需要它。

### 2. 配置数据库连接

运行迁移前，先准备本地运行时配置：

Linux：

```bash
cp config/development.toml.example config/development.toml
# 继续编辑 config/development.toml，把 database.password 改成当前本地 PostgreSQL 的真实密码或非空开发值，并填写 qq_bot.api_url、qq_bot.public_qq_number、qq_bot.bearer_token 和 public_site.domain / public_site.port
```

Windows 原生：

```powershell
Copy-Item config/development.toml.example config/development.toml
# 继续编辑 config/development.toml，把 database.password 改成当前本地 PostgreSQL 的真实密码，并填写 qq_bot.api_url、qq_bot.public_qq_number、qq_bot.bearer_token 和 public_site.domain / public_site.port
```

生产环境则应先复制：

```bash
cp config/production.toml.example config/production.toml
```

开发模板中的数据库段如下：

```toml
# config/development.toml
[database]
type = "postgres"
host = "127.0.0.1"
port = 5432
username = "postgres"
password = "CHANGE-THIS-LOCAL-POSTGRES-PASSWORD"
database = "electricity_dev"
max_connections = 5
min_connections = 1
connection_timeout = 30
```

```toml
# config/production.toml.example
[database]
type = "postgres"
host = "db.example.internal"
port = 5432
username = "postgres"
password = ""
password_file = "/run/secrets/app_database_password"
database = "electricity_pro"
max_connections = 20
min_connections = 5
connection_timeout = 30
```

开发环境迁移命令会复用应用本身的配置加载逻辑，并拒绝连接非本地数据库。Linux 开发可以连接系统 PostgreSQL，也可以连接映射到 `127.0.0.1:5432` 的 Docker PostgreSQL 容器。

## 创建迁移

### 1. 生成迁移文件

```bash
diesel migration generate create_users
```

这会在 `migrations/` 目录下创建两个文件：
- `up.sql`: 应用迁移的SQL
- `down.sql`: 回滚迁移的SQL

### 2. 编写迁移SQL

**up.sql** (应用迁移):
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
```

**down.sql** (回滚迁移):
```sql
DROP TABLE IF EXISTS users;
```

### 3. 运行迁移

```bash
# 开发环境
cargo run --bin migrate

# 生产环境
cargo run --bin migrate -- production
```

### 4. 验证Schema生成

迁移成功后，Diesel会自动更新 `src/infrastructure/database/schema.rs`：

```rust
diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
```

## 迁移管理

### 查看迁移状态

```bash
# 设置环境变量后查看
$env:DATABASE_URL = "postgres://user:pass@host:port/db"
diesel migration list
```

### 回滚迁移

```bash
# 回滚最后一次迁移
cargo run --bin migrate -- --revert

# 回滚生产环境
cargo run --bin migrate -- production --revert
```

### 重新运行所有迁移

```bash
# 1. 回滚所有迁移
cargo run --bin migrate -- --revert  # 多次执行直到全部回滚

# 2. 重新运行
cargo run --bin migrate
```

## 最佳实践

### 1. 迁移文件命名

使用描述性名称：
```bash
diesel migration generate create_rooms_table
diesel migration generate add_send_flag_to_rooms
diesel migration generate create_rooms_index
```

### 2. 原子性迁移

每个迁移应该是原子的、可回滚的：
- ✅ 一个迁移创建一个表
- ✅ 一个迁移添加一个索引
- ❌ 一个迁移创建多个不相关的表

### 3. 数据迁移

如果需要迁移数据，使用SQL：
```sql
-- up.sql
ALTER TABLE rooms ADD COLUMN new_field VARCHAR(100);
UPDATE rooms SET new_field = old_field WHERE old_field IS NOT NULL;
ALTER TABLE rooms DROP COLUMN old_field;
```

### 4. 索引优化

为常用查询字段创建索引：
```sql
-- 普通索引
CREATE INDEX idx_rooms_roomid ON rooms(roomid);

-- 部分索引（只索引特定条件的行）
CREATE INDEX idx_rooms_send_flag ON rooms(send_flag) WHERE send_flag = TRUE;

-- 复合索引
CREATE INDEX idx_rooms_roomid_created_at ON rooms(roomid, created_at);
```

### 5. 触发器和函数

PostgreSQL触发器示例：
```sql
-- 创建触发器函数
CREATE OR REPLACE FUNCTION update_send_flag()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.electricity_fee > NEW.threshold THEN
        NEW.send_flag := TRUE;
    END IF;
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 创建触发器
CREATE TRIGGER trigger_update_send_flag
BEFORE INSERT OR UPDATE OF electricity_fee, threshold ON rooms
FOR EACH ROW
EXECUTE FUNCTION update_send_flag();
```

## 故障排查

### 问题1: 迁移二进制无法连接数据库

**错误**:
```
❌ 迁移失败: connection to server at ...
```

**解决**:
1. 检查数据库服务是否运行
2. 验证当前环境对应的运行时配置文件中的连接信息
3. 确认网络连接、防火墙和容器网络设置

### 问题2: 配置文件未找到

**错误**:
```
❌ 加载配置失败: 缺少运行时配置文件 config/development.toml
```

**解决**:
确保在项目根目录执行命令，并先从对应模板复制出当前环境对应的运行时配置文件：

- 开发环境：`config/development.toml.example -> config/development.toml`
- 生产环境：`config/production.toml.example -> config/production.toml`

### 问题3: 生成迁移时 diesel 命令未找到

**错误**:
```
diesel: command not found
```

**解决**:
```bash
cargo install diesel_cli --no-default-features --features postgres
```

### 问题4: Schema未更新

**问题**: 运行迁移后 `schema.rs` 没有更新

**解决**:
检查 `diesel.toml` 配置：
```toml
[print_schema]
file = "src/infrastructure/database/schema.rs"

[migrations_directory]
dir = "migrations"
```

## 环境变量说明

`migrate` 工具直接读取当前环境对应的 TOML 配置，并使用解析后的数据库连接串建立 PostgreSQL 连接；不会要求外部 `.env` 文件，也不会要求全局设置 `DATABASE_URL`。

## 多环境管理

### 开发环境
```bash
cargo run --bin migrate
```
使用从 `config/development.toml.example` 复制得到的 `config/development.toml`

### 生产环境
```bash
cargo run --bin migrate -- production
```
使用从 `config/production.toml.example` 复制得到的 `config/production.toml`

## 参考资源

- [Diesel官方文档](https://diesel.rs/)
- [Diesel迁移指南](https://diesel.rs/guides/getting-started)
- [PostgreSQL触发器文档](https://www.postgresql.org/docs/current/sql-createtrigger.html)
