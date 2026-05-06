# 构建配置指南

## 📋 目录

- [概述](#概述)
- [PostgreSQL 配置](#postgresql-配置)
- [编译优化配置](#编译优化配置)
- [常见问题](#常见问题)

---

## 概述

本项目采用跨平台 Cargo 配置：仓库级 `.cargo/config.toml` 只保存稳定的编译优化项，不保存任何开发机私有路径。PostgreSQL / OpenSSL 的库路径由当前平台的系统包、标准安装路径或用户环境变量提供。

### 配置架构

```
.cargo/config.toml    →  跨平台稳定编译项（不写宿主机私有路径）
         ↓
系统包 / 用户环境变量 →  libpq / OpenSSL 链接信息
         ↓
    build.rs          →  Windows 标准路径辅助检测
         ↓
    编译系统           →  最终构建
```

**设计原则**:
1. ✅ **仓库配置可迁移**: 不把 `C:\...`、`/home/...` 等机器私有路径写入版本控制
2. ✅ **Linux 使用系统包**: 通过 `libpq-dev`、`libssl-dev`、`pkg-config` 提供编译依赖
3. ✅ **Windows 使用local environment环境变量**: 非标准安装路径通过用户环境变量指定
4. ✅ **Docker 独立静态构建**: 镜像构建走 `static-build` feature，不依赖开发机路径

---

## PostgreSQL 配置

### Linux 推荐方式：系统包

Linux 开发环境应安装客户端、头文件和编译工具；PostgreSQL / Redis 服务本身可以是系统服务，也可以是映射到本地端口的 Docker 容器：

```bash
sudo apt-get update
sudo apt-get install -y build-essential libpq-dev libssl-dev pkg-config postgresql-client redis-tools
```

验证：

```bash
pg_config --version
psql --version
redis-cli --version
cargo check
```

Linux 下不要在 `.cargo/config.toml` 写入 `PQ_LIB_DIR` 或 `OPENSSL_DIR`。这些变量会被 Cargo 注入到所有平台，一旦写入 Windows 路径，Linux 构建会被错误路径污染。
运行 `cargo run --bin migrate` 不要求安装 Diesel CLI；生成新 migration 或手动刷新 schema 时再安装 `diesel_cli`。

### Windows 原生推荐方式：标准安装路径

如果 PostgreSQL 安装在标准路径，通常无需额外配置：

#### Windows 支持的路径

```
C:\Program Files\PostgreSQL\16\lib
C:\Program Files\PostgreSQL\15\lib
C:\Program Files\PostgreSQL\14\lib
```

**使用步骤**:

```powershell
# 直接编译即可
cargo build

# 查看检测结果
# 输出: "自动检测到 PostgreSQL: C:\Program Files\PostgreSQL\16\lib"
```

---

### Windows 原生手动配置（非标准路径）

如果 PostgreSQL 安装在自定义位置，不要修改仓库级 `.cargo/config.toml`。请设置当前用户或当前 shell 的环境变量，避免把私有路径提交到仓库。

#### 配置选项

**选项1: 直接指定 lib 目录**

```powershell
$env:PQ_LIB_DIR = "D:\PostgreSQL\lib"
```

**选项2: 指定安装根目录（推荐）**

```powershell
$env:POSTGRES_HOME = "D:\PostgreSQL"
```

> `build.rs` 会自动在 `POSTGRES_HOME/lib` 查找库文件

#### 配置步骤

1. **定位 PostgreSQL 安装路径**:

```powershell
# Windows: 查找 psql 位置
where.exe psql
# 输出: C:\custom\PostgreSQL\bin\psql.exe
# 则 lib 目录为: C:\custom\PostgreSQL\lib
```

```bash
# Linux: 查看系统包提供的 libpq 信息
which pg_config
pg_config --libdir
```

2. **设置当前 shell 环境变量**:

```powershell
$env:PQ_LIB_DIR = "C:\custom\PostgreSQL\lib"
```

3. **验证配置**:

```powershell
# 清理缓存
cargo clean

# 重新编译
cargo build

# 查看构建输出，确认路径正确
```

---

### ⚙️ 配置优先级

链接信息按以下顺序进入构建：

```
1. 显式环境变量 PQ_LIB_DIR / POSTGRES_HOME（主要用于 Windows 非标准路径）
   ↓
2. Windows 标准安装路径辅助检测
   ↓
3. Linux 系统包和 pkg-config
   ↓
4. Docker static-build feature 的 bundled/vendored 构建
```

**边界**: `.cargo/config.toml` 不能承载任何机器私有路径。需要长期记录的只有跨平台稳定项，例如 `target-cpu=native`。

---

## 编译优化配置

### SIMD 优化

项目已启用 SIMD 优化（sonic-rs 需要）：

```toml
[build]
rustflags = ["-C", "target-cpu=native"]
```

### Release 优化

Release 模式配置（Cargo.toml）：

```toml
[profile.release]
opt-level = 3           # 最高优化级别
lto = "fat"             # 完整 LTO
codegen-units = 1       # 单编译单元
strip = true            # 删除调试符号
```

**效果**:
- ✅ 二进制体积减少 ~40%
- ✅ 运行速度提升 ~20%
- ✅ JSON 序列化性能提升 ~3x（sonic-rs SIMD）

---

## 常见问题

### ❌ 问题1: 链接错误 - 无法解析外部符号

**症状**:
```
error LNK2019: 无法解析的外部符号 gettimeofday
error LNK2019: 无法解析的外部符号 pg_b64_encode
```

**原因**: PostgreSQL 路径配置错误，或指向了不完整的静态库（如 vcpkg）

**解决方案**:

1. **检查系统环境变量**:

```powershell
# Windows
[System.Environment]::GetEnvironmentVariable('PQ_LIB_DIR', 'Machine')
[System.Environment]::GetEnvironmentVariable('PQ_LIB_DIR', 'User')

# 如果输出了错误的路径（如 vcpkg），删除它
[System.Environment]::SetEnvironmentVariable('PQ_LIB_DIR', $null, 'Machine')
```

2. **清理仓库级路径污染**:

确认 `.cargo/config.toml` 中没有 `PQ_LIB_DIR`、`OPENSSL_DIR`、`OPENSSL_LIB_DIR` 或 `OPENSSL_INCLUDE_DIR`。这些变量只能存在于开发机用户环境变量或当前 shell。

3. **清理并重新编译**:

```powershell
cargo clean
Remove-Item Env:PQ_LIB_DIR -ErrorAction SilentlyContinue
cargo build
```

---

### ❌ 问题2: build.rs 未找到 PostgreSQL

**症状**:
```
warning: 未找到 PostgreSQL 库目录，请设置 PQ_LIB_DIR 或 POSTGRES_HOME 环境变量
```

**解决方案**:

1. **确认 PostgreSQL 已安装**:

```powershell
where.exe psql
```

```bash
which psql
pg_config --version
```

2. **手动配置路径**（见上文"方式2：手动配置"）

---

### ❌ 问题3: 默认库 LIBCMT 冲突

**症状**:
```
warning LNK4098: 默认库"LIBCMT"与其他库的使用冲突
```

**原因**: 使用了 vcpkg 的静态库，与 MSVC 运行时冲突

**解决方案**: 使用 PostgreSQL 官方动态库（自动检测会优先使用官方库）

---

### ❌ 问题4: libintl.lib 无法打开

**症状**:
```
LINK : fatal error LNK1181: 无法打开输入文件"libintl.lib"
```

**原因**: PostgreSQL 库目录不完整

**解决方案**:

1. **验证 lib 目录内容**:

```powershell
Get-ChildItem 'C:\Program Files\PostgreSQL\16\lib' -Filter *.lib

# 应该包含:
# libpq.lib
# libpgcommon.lib
# libpgport.lib
# libintl.lib
# libssl.lib
# libcrypto.lib
```

2. **如果缺少文件，重新安装 PostgreSQL**

---

## 📚 相关文档

- [快速启动指南](./QUICKSTART.md) - 完整的开发环境搭建
- [架构设计](../architecture/ARCHITECTURE.md) - 项目架构说明
- [API 参考](../api/API_REFERENCE.md) - API 文档

---

## 🔗 外部资源

- [PostgreSQL 官方下载](https://www.postgresql.org/download/)
- [Rust 工具链安装](https://www.rust-lang.org/tools/install)
- [Diesel 文档](https://diesel.rs/)

---

**最后更新**: 2026-05-05
**维护者**: Electricity Monitor Team
