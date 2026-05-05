---
type: semantic
status: verified
scope: 运行时配置与环境
updated_at: 2026-04-17
verified_at: 2026-04-17
sources:
  - src/config/app.rs
  - config/development.toml.example
  - config/production.toml.example
  - scripts/backend-checks.ps1
  - scripts/backend-checks.sh
summary: 运行时配置加载顺序、环境语义、关键依赖和 fail-fast 约束
---

# Electricity Monitor 运行时配置与环境

## 背景

- 配置入口在 `src/config/app.rs`。
- 运行时配置由活动环境对应的 `.toml` 文件和 `APP__...` 覆盖链共同决定。

## 稳定事实

- 加载顺序固定为：
  1. 当前环境对应的运行时配置文件：`config/development.toml` 或 `config/production.toml`
  2. 环境变量 `APP__<SECTION>__<KEY>`
- 默认环境名是 `development`。
- `config/` 目录下只能保留一个运行时 `.toml` 文件，文件名必须是 `development.toml` 或 `production.toml`，并与 `APP_ENV` 保持一致。
- `config/development.toml` 与 `config/production.toml` 都不纳入版本控制；运行前先从对应模板复制，再在目标环境中补齐真实值。
- 缺少当前环境对应的运行时配置文件时，应用会明确报错并提示从模板复制。

## 环境语义

- `development` 环境只能连接本地 PostgreSQL 和本地 Redis；本地依赖可以是系统服务，也可以是映射到 `127.0.0.1` 的 Docker 容器。
- `production` 环境允许远端数据库和 Redis，但敏感值必须通过 `*_FILE` 链路注入。
- `APP_ENV` 既决定环境校验与 fail-fast 规则，也要求运行时配置文件名与之保持一致。
- `RUST_LOG` 的优先级高于配置文件中的 `logging.level`。

## 关键依赖与约束

- PostgreSQL 是主数据存储，Redis 负责验证码、限流、缓存和后台任务协作。
- `APP__SECTION__KEY` 与 `APP__SECTION__KEY_FILE` 都是正式支持的覆盖方式。
- 不要启用全局 `try_parsing(true)`；当前配置链路依赖保留前导零字符串。
- 开发环境要求在复制出来的 `config/development.toml` 中显式填写 `database.password`，不能依赖隐式环境变量；如果本地 Docker PostgreSQL 使用 trust 认证，也必须把占位符替换为非空开发值。
- `development` 环境下如果 `database.password` 为空或仍是模板占位值，应用会在配置阶段直接失败。
- `production` 环境缺少 `jwt.secret_file`、`database.password_file` 或 `qq_bot.bearer_token_file` 时会 fail-fast。

## 与测试相关的环境变量

- `RUN_INTEGRATION_TESTS=1`：显式开启依赖本地 PostgreSQL / Redis 的源码内环境型测试。
- `REDIS_HOST` / `REDIS_PORT`：覆盖 Redis 测试连接。
- `RUN_EXTERNAL_INTEGRATION_TESTS=1`：显式开启依赖外部网络的真实测试。

## 相关决策

- `../../decisions/runtime-config-uses-environment-named-toml.md`
