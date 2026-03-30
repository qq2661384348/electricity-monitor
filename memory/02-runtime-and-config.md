# Electricity Monitor 仓库记忆：运行时、配置与环境变量

## 配置加载规则
- 配置入口在 `src/config/app.rs`。
- 加载顺序：
  1. `config/default.toml`
  2. `config/{APP_ENV}.toml`
  3. 环境变量 `APP__<SECTION>__<KEY>`
- 默认环境名是 `development`。

## 当前环境语义
- `development` 环境被明确限制为只能连接本地 PostgreSQL 和本地 Redis。
- `production` 环境仍允许远端数据库配置，但敏感值必须通过 Compose secrets 对应的 `*_FILE` 链路注入。
- 日志优先级：`RUST_LOG` 高于配置文件 `logging.level`。

## 关键运行依赖
- PostgreSQL：主数据存储。
- Redis：验证码、限流、缓存、后台任务协作。
- 外部房间树接口：`room_sync.crawler.api_url`
- 外部电费查询接口：`electricity_fetcher.api_url`
- NapCat HTTP 机器人服务：`qq_bot.api_url` 默认指向local environment action endpoint，占位值为 `http://127.0.0.1:3000/send_private_msg`；部署时按实际地址覆盖。

## 关键环境变量记忆
- `APP_ENV`
- `RUST_LOG`
- `APP__DATABASE__HOST`
- `APP__DATABASE__PORT`
- `APP__DATABASE__USERNAME`
- `APP__DATABASE__PASSWORD_FILE`
- `APP__DATABASE__DATABASE`
- `APP__REDIS__HOST`
- `APP__REDIS__PORT`
- `APP__QQ_BOT__API_URL`
- `APP__JWT__SECRET_FILE`
- `APP__QQ_BOT__BEARER_TOKEN`
- `APP__QQ_BOT__BEARER_TOKEN_FILE`
- `APP__LOGGING__LEVEL`

## 环境变量链路的已验证结论
- `APP__SECTION__KEY` 双下划线嵌套覆盖可用。
- `APP__SECTION__KEY_FILE` 同样可用于 secret file 覆盖。
- 数值和布尔类型字段可正确反序列化，不需要额外启用全局 `try_parsing(true)`。
- 不能启用全局 `try_parsing(true)`，否则会破坏带前导零的字符串型配置值。
- 公开仓库只保留 loopback/placeholder 形式的机器人地址；真实地址与 token 必须通过运行时配置或 secret file 注入，不写回仓库。
- `production` 环境缺少 `jwt.secret_file`、`database.password_file` 或 `qq_bot.bearer_token_file` 时会 fail-fast。

## 测试链路记忆
- 数据库集成测试通过 `RUN_INTEGRATION_TESTS=1` 显式开启。
- Redis 相关测试通过 `RUN_INTEGRATION_TESTS=1` 或 `REDIS_HOST/REDIS_PORT` 启用。
- 当前仓库完整 Rust 测试在最近一次验证中为 `111 passed`。
- `tests/runtime/release_readiness_test.rs` 当前覆盖 `/api/health`、`/api/health/db` 和静态资源入口 `/`。
