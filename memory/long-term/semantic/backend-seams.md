---
type: semantic
status: verified
scope: 后端可维护性接缝
updated_at: 2026-05-06
verified_at: 2026-05-06
sources:
  - src/modules/auth/api/middleware.rs
  - src/handlers/auth.rs
  - src/handlers/binding.rs
  - src/domain/services/rate_limiter.rs
  - src/modules/room/application/mod.rs
  - src/infrastructure/cache/cache_manager.rs
  - src/domain/services/notification_gate.rs
  - src/domain/services/notification_service.rs
summary: 后端鉴权、验证码限流、房间授权、缓存、通知域和模块化迁移接缝
---

# Electricity Monitor 后端可维护性接缝

## 鉴权接缝

- `src/modules/auth/` 是鉴权边界真源，负责 claims 解析、`Actor` 映射和认证中间件。
- `src/middleware/auth.rs` 仅保留兼容 facade / `UserContext` 投影，不应重新变成主逻辑入口。
- 管理员已改走与普通用户一致的验证码登录链路。
- `src/handlers/auth.rs` 当前承接双登录模式的 HTTP 兼容入口：`login_mode=qq|email` + `identifier` 为新契约，`qq_number` 和 `email` 只作为兼容字段。
- `UserRepository` 负责按 `login_provider` 隔离 QQ 与邮箱账号，避免 handler 直接绕过 provider 维度查询用户。
- JWT claims 现在显式区分 `token_kind=access|refresh`；受保护接口只接受 access token，`/api/auth/refresh` 只接受 refresh token。
- refresh token 只通过 `Set-Cookie` / `Cookie` 往返，JSON 响应不再包含 refresh token 字段。
- `src/handlers/auth.rs` 负责签发 access token、轮换 refresh cookie 与 logout 清理 cookie 的 HTTP 契约；不要在其他 handler 重新实现这套逻辑。
- `src/domain/services/rate_limiter.rs` 是认证公开入口的 Redis 固定窗口配额接缝；`send-verification-code` 在这里复用全局、客户端来源和目标维度限流，不应在 handler 内自行拼接 Redis 限流 key。

## 缓存接缝

- `src/infrastructure/cache/cache_manager.rs` 是统一缓存入口，覆盖 room、user、binding 和 electricity。
- `src/state.rs` 持有正式接入的 `CacheManager`，主链路已经在 `auth`、`path_tree`、`room`、`binding` 相关读取与失效点使用缓存。
- 若继续扩展缓存，优先补后台更新链和通知相关读取链，不要重新引入第二套缓存入口。

## 通知域接缝

- `NotificationGate` 负责去重、防抖观察期、恢复状态、内存 + 数据库双写持久化，以及周期清理与恢复任务。
- `NotificationService` 负责查房间、查绑定、查用户、构建消息、按登录渠道分发到 QQClient 或 EmailDelivery、调 gate，以及并发控制与统计。
- 邮箱通知模板在 `src/infrastructure/email/templates.rs`，应与机器人通知内容保持语义一致但使用 HTML 邮件结构。
- 通知域内部已经包含多个子职责，后续拆分应围绕这两个接缝继续推进。

## Handler 与模块化接缝

- `room`、`path_tree`、`room_sync` 的复杂编排已经开始下沉到 `src/modules/*/application`。
- 房间详情读取（包括 id、roomid、path、hash 入口）统一由 `RoomAccessUseCase` 做访问控制；handler 不应直接绕过 use case 读取完整房间电费和阈值。
- 普通用户创建房间绑定前必须通过管理员签发的 `binding_proof` 校验；已有绑定仍是 `RoomAccessUseCase` 的授权事实，因此清理历史异常绑定应通过数据审计处理，而不是放宽房间读取 guard。
- 旧 handler 热点主要集中在 `binding` 与 `auth`，后续应继续往模块应用层收敛。

## 外部 HTTP 接缝

- `src/infrastructure/external/` 提供统一 `reqwest` 客户端构造与 HTTP 状态错误映射。
- `electricity`、`room_sync crawler`、`qq_client` 已接入这条统一入口。
- `electricity` 电费抓取客户端必须保留 HTTPS 证书校验；测试或开发调试不得把 `danger_accept_invalid_certs=true` 带回生产路径。
- 电费全量抓取会覆盖生产库全部 active roomid；`RoomBatchFetcher` 必须通过 `buffer_unordered(self.max_concurrent)` 做流式背压，不能提前为所有房间 `tokio::spawn`，`ElectricityFetcherService` 的定时入口必须跳过未完成的上一轮任务，避免批处理内存高水位和外部 API 压力叠加。
- 新增外部 HTTP 依赖时，应优先复用这条接缝，而不是各模块自行创建 `reqwest::Client`。
