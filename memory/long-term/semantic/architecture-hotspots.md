---
type: semantic
status: verified
scope: 前后端结构热点
updated_at: 2026-04-17
verified_at: 2026-04-17
sources:
  - src/infrastructure/repositories/room_repository.rs
  - src/domain/services/notification_gate.rs
  - frontend/src/pages/DashboardPage.tsx
  - frontend/src/services/api.ts
summary: 当前仍需持续关注的后端、前端和横向结构热点
---

# Electricity Monitor 架构热点

## 后端热点

- `src/infrastructure/repositories/room_repository.rs` 仍承载过多职责，混合了查询、更新、同步日志、缓存支持和恢复时间持久化。
- `src/domain/services/notification_gate.rs` 与 `src/domain/services/notification_service.rs` 体量较大，通知域边界仍值得继续拆分。
- `src/handlers/binding.rs` 与 `src/handlers/auth.rs` 仍有旧式编排痕迹，后续应继续往 `application/usecase` 收敛。

## 前端热点

- `frontend/src/pages/DashboardPage.tsx` 仍是高耦合页面容器，集中承担认证门禁、查询结果转换、事件处理和多个 modal 状态编排。
- `frontend/src/components/BindRoomModal.tsx` 仍是流程型大组件，适合继续拆分状态、查询与展示。
- `frontend/src/services/api.ts` 仍作为兼容 facade 聚合多类接口，是后续继续瘦身的候选点。
- `frontend/src/features/dashboard/hooks/useDashboardData.ts` 仍保留迁移痕迹，说明 dashboard 数据层尚未完全收敛。

## 横向关注点

- 需要优先做边界收敛，而不是只做风格化重写。
- 文档与代码真源仍可能漂移，尤其是 `docs/guides/TECHNICAL_DEBT.md` 这类偏历史性的说明文件。
