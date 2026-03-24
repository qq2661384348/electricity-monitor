# Electricity Monitor 仓库记忆：第三轮可维护性接缝

## 鉴权接缝
- `src/middleware/auth.rs` 当前采用“双轨鉴权”：
  - 管理员：固定 `admin_token`
  - 普通用户：JWT Claims
- 这让鉴权逻辑简单可用，但也把“管理员身份模型”和“用户身份模型”混在一个中间件中。
- 后续若升级认证体系，应优先把：
  - token 解析
  - 角色判定
  - 请求上下文注入
  拆成更稳定的边界，而不是继续在 handler 中扩散权限判断。

## 缓存接缝
- `src/infrastructure/cache/cache_manager.rs` 已经形成统一缓存管理器雏形，包含：
  - Room
  - User
  - Binding
  - Electricity
- 但 `src/state.rs` 中 `cache_manager` 仍为 `None` 初始化，说明缓存架构是“设计存在、接入不完整”的状态。
- 这类“半接入基础设施”很重要，后续架构升级应明确：
  - 要么正式接入主链路
  - 要么收敛/移除，避免长期保持悬空能力

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
- 当前多个 handler 直接实例化 Repository 并执行权限判断、查询编排、响应拼装。
- 这说明“application service / use case 层”在 HTTP 层与仓储层之间仍不够稳定。
- 后续若要提升可维护性，适合把复杂 handler 的编排逻辑向更窄的用例层收敛。

## 前端可复用资产接缝
- `frontend/src/components/ui/comic-modal/` 已经采用 compound components 模式，是较成熟的可复用 UI 资产。
- 这说明前端并不是完全缺少设计系统，而是“页面容器逻辑过重、基础组件资产已有雏形”。
- 后续前端升级应优先复用这类已有基础设施，而不是重做一套 UI 组件模式。
