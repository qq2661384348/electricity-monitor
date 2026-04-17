---
type: semantic
status: verified
scope: 前端可维护性接缝
updated_at: 2026-04-17
verified_at: 2026-04-17
sources:
  - frontend/src/App.tsx
  - frontend/src/shared/api/http-client.ts
  - frontend/src/features
  - frontend/src/entities
summary: 前端启动、会话、HTTP、page/feature/entity 和可复用 UI 接缝
---

# Electricity Monitor 前端可维护性接缝

## 启动与会话接缝

- 前端启动骨架固定为 `frontend/src/main.tsx` -> `frontend/src/App.tsx` -> `frontend/src/routes.tsx`。
- 浏览器端只在内存持有 access token。
- refresh token 只存在于后端签发的 HTTPOnly Cookie，不通过 JSON 暴露给前端代码。
- 页面刷新后的会话恢复统一走 `/api/auth/refresh`。
- 并发 401 回收敛到单个 in-flight refresh promise，避免同一轮失效触发刷新风暴。

## HTTP 与兼容接缝

- `frontend/src/shared/api/http-client.ts` 是真实 HTTP client 真源，统一处理 `/api` 前缀、`withCredentials`、Bearer token 注入、refresh 和 401 重放。
- `frontend/src/services/api.ts` 只是兼容 facade，不应重新演化成新的 API 真源。
- 新增接口优先落在 `shared/api`、`entities/*/api` 或 `features/*/api`，不要直接在页面或组件里散落 Axios 调用。

## 页面 / feature / entity 接缝

- 页面层优先承担容器和布局装配职责，复杂流程下沉到 `features/`。
- `features/` 负责流程级 `api / model / ui / index.ts` 组织、query invalidation、mutation 编排和页面交互状态。
- `entities/` 负责单领域 API、轻量领域转换和稳定公共出口，不承接页面状态、路由跳转或跨实体编排。

## 可复用 UI 接缝

- `frontend/src/components/ui/comic-modal/` 已采用 compound components 模式，是当前较成熟的可复用 UI 资产。
- 前端后续升级应优先复用现有基础设施，而不是重做一套新的 UI 组件模式。
