# 后端模块 AGENTS

## 作用范围

- 本文件补充根目录 `AGENTS.md` 与 `src/AGENTS.md`，只约束 `src/modules/` 里的模块化迁移代码。

## 真源与入口

- `src/modules/` 是后端从旧式 `handlers + domain services + repositories` 逐步收敛到稳定边界的主迁移层。
- 模块默认采用 `api / application / domain / infrastructure` 四层拆分：
  - `api/` 负责中间件、HTTP 适配或模块对外入口。
  - `application/` 负责 use case、跨依赖编排和事务性流程。
  - `domain/` 负责模块内部身份模型、规则对象和纯业务语义。
  - `infrastructure/` 负责模块私有的 repository adapter、credential resolver 或外部依赖封装。
- `auth` 是当前身份与鉴权边界真源；`modules/auth/domain` 定义 `Actor/Claims`，`modules/auth/api/middleware.rs` 是认证中间件真源。
- `src/middleware/auth.rs` 只是兼容 facade；改鉴权主逻辑时优先改 `modules/auth/`。
- `room` 与 `room_sync` 是当前活跃迁移接缝；新增房间访问和同步编排优先落在这两个模块的 `application/`。

## 首选接缝

- 新 use case 优先提供类似 `from_state(&AppState)` 的构造入口，统一复用 `AppState` 中的 pool、cache 和共享服务。
- 需要跨层编排时，只在 `application/` 里组合 repository、cache、domain rule 和外部服务。
- `domain/` 保持和 Axum、`AppState`、HTTP 细节解耦；`infrastructure/` 不要反向依赖路由或 handler。

## 边界与禁止项

- 不要把已经迁进模块的编排逻辑再搬回旧 `src/handlers/`。
- 不要在 handler 里重复解析 `Actor`、`Claims` 或权限语义；优先走 `modules/auth/` 提供的接缝。
- 不要让 `application/` 直接承担页面式 DTO 拼装或 HTTP 响应格式控制，那是 `api/` 的职责。
- 不要因为临时需求跳过模块公共出口，直接在多个旧文件里复制同一段流程。

## 最小验证

- 改动模块边界后，至少确认 `src/modules/mod.rs`、对应模块子目录和引用路径保持一致。
- 涉及鉴权模块时，至少运行 `cargo test --test auth_integration_test`。
- 涉及房间或同步主链路的结构调整时，运行 `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`，并按影响范围补 `cargo test --lib` 或 `cargo test --test release_readiness_test`。
