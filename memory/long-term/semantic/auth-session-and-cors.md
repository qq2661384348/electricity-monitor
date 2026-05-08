---
type: semantic
status: verified
scope: 鉴权会话、CORS 与管理员提升规则
updated_at: 2026-05-08
verified_at: 2026-05-08
sources:
  - src/handlers/auth.rs
  - src/handlers/binding.rs
  - src/routes/binding.rs
  - src/config/captcha.rs
  - src/config/verification.rs
  - src/domain/services/rate_limiter.rs
  - src/domain/models/user.rs
  - src/infrastructure/repositories/user_repository.rs
  - src/modules/auth/api/middleware.rs
  - src/modules/auth/application/actor_resolver.rs
  - src/modules/room/application/mod.rs
  - src/handlers/path_tree.rs
  - src/config/cors.rs
  - docs/api/API_REFERENCE.md
summary: JWT 类型边界、cookie 会话契约、验证码发送配额、CORS 白名单、管理员提升规则、绑定创建和房间详情授权边界
---

# Electricity Monitor 鉴权会话、CORS 与权限提升规则

## 鉴权边界

- `src/modules/auth/` 是鉴权边界真源，负责 claims 解析、`Actor` 映射和认证中间件。
- `src/middleware/auth.rs` 仅保留兼容 facade / `UserContext` 投影，不应重新变成主逻辑入口。
- 登录身份显式区分 `login_provider=qq|email`；`users.qq_number` 和 `users.email` 按渠道互斥，`user_room_bindings.user_id` 继续作为账号数据隔离边界。
- `/api/auth/send-verification-code` 必须先消费 `/api/captcha/verify` 签发的一次性 `captcha_token`，缺失、过期或重复使用时不得调用 QQ 机器人或 SMTP 邮件发送器发送验证码。
- `/api/auth/send-verification-code` 使用 Redis 固定窗口配额限制公开发送入口：发送前先检查全局配额和连接层 peer IP 配额，消费 captcha 后再检查 `provider:identifier` 目标配额；触发配额时返回 429，不继续调用 QQ 或 SMTP。
- 当前应用层没有受信代理配置，客户端来源配额不再信任 `CF-Connecting-IP`、`X-Real-IP`、`X-Forwarded-For` 等可伪造转发头；如需真实公网客户端粒度，应在边缘层完成限流，或先引入明确的 trusted proxy 配置。
- `/api/captcha/verify` 签发的 `captcha_token` 有效期由 `captcha.token_expire_seconds` 控制；登录验证码长度和 Redis 有效期分别由 `verification.code_length` 与 `verification.expire_seconds` 控制，Redis key 必须带 `qq` / `email` 渠道前缀避免跨模式混淆。
- `/api/auth/verify-and-login` 只接受与 `verification.code_length` 完全一致的数字验证码，不应在前端或后端继续硬编码 6 位长度。
- JWT claims 显式区分 `token_kind=access|refresh`；受保护接口只接受 access token，`/api/auth/refresh` 只接受 refresh token。
- 受保护接口解析 access token 后会通过 `ActorResolver` 重新读取当前用户记录；`is_active=false` 的旧 token 返回 401，角色授权以数据库当前 `role` 为准，降权后的旧 admin access token 不再拥有管理员权限。
- JWT `sub` 使用 `provider:identifier` 形式，避免 QQ 号与邮箱地址在不同登录渠道下同值混淆。
- `src/handlers/auth.rs` 负责签发 access token、轮换 refresh cookie 与 logout 清理 cookie 的 HTTP 契约；不要在其他 handler 重新实现这套逻辑。
- `/api/bindings` 以当前登录账号为个人绑定主体；创建绑定必须通过 access token Bearer 认证，但普通用户和管理员账号都不再需要额外绑定码。
- `/api/bindings` 只创建当前登录账号自己的绑定；绑定创建后才会成为 `RoomAccessUseCase` 允许普通用户读取房间详情的授权事实。
- `/api/rooms/by-path` 与 `/api/rooms/by-hash` 是房间详情读取入口，必须通过 `RoomAccessUseCase::ensure_room_access` 约束为管理员或已绑定用户；绑定前路径树只允许返回叶子节点 `roomid` 这类最小绑定标识，不返回电费余额或阈值。

## 会话模型

- refresh token 只通过 `Set-Cookie` / `Cookie` 往返，JSON 响应不再包含 refresh token 字段。
- refresh cookie 契约由 `auth.refresh_cookie_secure`、`auth.refresh_cookie_same_site` 和 `auth.refresh_expiration_hours` 控制。
- 当 `SameSite=None` 时必须同时启用 `Secure`。
- 浏览器端只在内存持有 access token，不应重新落回本地持久化存储。

## CORS 规则

- `cors.allowed_origins` 使用逗号分隔字符串维护前端 Origin 白名单，便于通过 `APP__CORS__ALLOWED_ORIGINS` 覆盖。
- 后端 CORS 以配置驱动白名单和 `allow_credentials(true)` 为准，不应回退到全局开放。
- `production` 环境会拒绝空白、`localhost` 或模板占位值形式的 `cors.allowed_origins`。

## 管理员提升规则

- `admin.default_qq_number` 只在值非空且不是模板占位值时才会授予 `admin`。
- `production` 环境会拒绝空白或占位值管理员 QQ。
- 管理员登录链路与普通用户一致，不再保留独立的绕行凭据入口。
- 邮箱登录本轮始终授予并保持 `user` 角色，不使用 `admin.default_qq_number`，也不启用邮箱管理员配置。

## 相关决策

- `../../decisions/browser-session-uses-memory-access-token-and-cookie-refresh.md`
