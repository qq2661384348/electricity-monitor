# Electricity Monitor 仓库记忆：第三轮可维护性接缝

## 鉴权接缝
- `src/modules/auth/` 已开始接管鉴权边界：
  - JWT Claims 中显式携带 `role`
  - middleware 先解析 Claims，再映射统一 `Actor`
  - `src/middleware/auth.rs` 仅保留兼容 facade / `UserContext` 投影
- 固定 `admin_token` 已移除，管理员改走和普通用户一致的验证码登录链路。

## 缓存接缝
- `src/infrastructure/cache/cache_manager.rs` 已经形成统一缓存管理器雏形，包含：
  - Room
  - User
  - Binding
  - Electricity
- `src/state.rs` 现已持有正式接入的 `CacheManager`，并在启动阶段预热 active rooms。
- 当前主链路已在 `auth` / `path_tree` / `room` / `binding` 相关读取与失效点使用缓存。
- 若继续扩展缓存，优先补后台更新链和通知相关读取链，不要重新引入第二套缓存入口。

## 通知域接缝
- `NotificationGate` 负责：
  - 去重
  - 防抖观察期
  - 恢复状态
  - 内存 + 数据库双写持久化
  - 周期清理与恢复任务
- `NotificationService` 负责：
  - 查房间
  - 查绑定
  - 查用户
  - 构建消息
  - 调 QQClient
  - 调 gate
  - 做并发控制与统计
- 这说明通知域内部实际上已经包含多个子职责，可作为后续拆分的重点候选。

## Handler 层接缝
- `room` / `path_tree` / `room_sync` 的复杂编排已经开始下沉到 `src/modules/*/application`。
- 当前仍需继续收敛的旧 handler 热点主要是 `binding` 与 `auth`。

## 外部 HTTP 接缝
- `src/infrastructure/external/` 已建立统一 `reqwest` 客户端构造与 HTTP 状态错误映射。
- 当前 `electricity`、`room_sync crawler`、`qq_client` 已接入这条统一入口。
- 后续新增外部 HTTP 依赖时，优先复用这条统一入口，而不是各模块自行 new `reqwest::Client`。

## 前端可复用资产接缝
- `frontend/src/components/ui/comic-modal/` 已经采用 compound components 模式，是较成熟的可复用 UI 资产。
- 这说明前端并不是完全缺少设计系统，而是“页面容器逻辑过重、基础组件资产已有雏形”。
- 后续前端升级应优先复用这类已有基础设施，而不是重做一套 UI 组件模式。
