# 后端源码 AGENTS

## 作用范围

- 本文件补充根目录 `AGENTS.md`，只约束 `src/` 里的后端代码协作方式。

## 真源与入口

- 后端进程入口是 `src/main.rs`，它只应调用 `bootstrap::app::run()`；不要把启动编排重新塞回 `main.rs`。
- 启动装配真源在 `src/bootstrap/`；配置初始化、日志、路由装配、运行时任务和 shutdown 都应优先在这里收敛。
- 路由编排真源在 `src/routes/`；`src/handlers/` 负责 HTTP 入参/出参适配，不是复杂业务编排真源。
- 新的业务编排优先进入 `src/modules/*/application`；`src/modules/` 是当前模块化迁移主线。
- 共享运行时资源真源在 `src/state.rs`；缓存、限流、连接池等共享依赖优先从 `AppState` 进入 use case。
- 配置模型真源在 `src/config/`，并受根级 `AGENTS.md` 的配置链路与 `_FILE` secrets 规则约束。
- 鉴权 HTTP 契约真源在 `src/handlers/auth.rs` 与 `src/modules/auth/`：access token 走 Bearer，refresh token 只走 HTTPOnly Cookie。
- 后端统一响应安全头真源在 `src/bootstrap/router.rs` 与 `src/middleware/security_headers.rs`；改运行时安全头时必须同步 `deploy/smoke.targets` 与 readiness test。

## 首选接缝

- 新增或重写复杂流程时，优先增加 use case 或 application service，而不是继续扩张 handler。
- 需要缓存时优先复用 `state.cache_manager` 和 `src/infrastructure/cache/`，不要再引入第二套缓存入口。
- 需要外部 HTTP 调用时优先复用 `src/infrastructure/external/` 的统一 client 和错误映射，不要在业务代码里散落 `reqwest::Client::new()`。
- 需要访问数据库或 Redis 时，优先沿用现有 repository、pool 和 state 注入方式，保持依赖方向稳定。
- 修改 JWT 解码、认证中间件或刷新流程时，必须保持 `token_kind=access|refresh` 边界，不要把两类 token 重新混用。

## 边界与禁止项

- 不要把新的编排逻辑、权限判断或跨仓储流程继续堆回 `src/handlers/`。
- 不要为了图快把生产 secrets、远端开发库地址或明文 token 写回 `config/development.toml`、`config/production.toml` 或 `*.toml.example`。
- 不要启用全局 `try_parsing(true)`；当前配置链路依赖保留前导零字符串。
- 不要把 refresh token 重新放回 JSON 响应或请求体。
- 不要绕开统一响应安全头链路，在单个 handler 或路由里散落第二套 header 写法。
- 不要在已有模块接缝的地方再造平行实现或临时 optimized 文件。

## 最小验证

- 仅改后端文档或上下文文件时，至少做路径与引用自检，确认这里提到的目录和入口仍存在。
- 涉及架构边界调整时，运行 `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`。
- 涉及后端行为改动时，按影响范围运行 `cargo test --lib`、`cargo test --test auth_integration_test`、`cargo test --test send_verification_code_integration_test`、`cargo test --test release_readiness_test`、`cargo clippy --all-targets -- -D warnings`、`cargo audit -q`。

## 同步要求

- 修改运行时、安全头、鉴权或 cookie 会话边界时，同步更新根 `AGENTS.md`、`docs/api/API_REFERENCE.md`、`memory/long-term/semantic/auth-session-and-cors.md` 与 `memory/long-term/procedural/testing-and-quality-gates.md`；若改变浏览器会话模型，再同步 `memory/decisions/browser-session-uses-memory-access-token-and-cookie-refresh.md`。
- 修改后端维护接缝、缓存、外部 HTTP 或 handler / module 分工时，同步更新 `memory/long-term/semantic/backend-seams.md`。
