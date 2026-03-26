# Electricity Monitor 仓库记忆：前端架构专项

## 当前前端技术基线
- 框架：React 19 + TypeScript + Vite 7。
- 路由：`createBrowserRouter`，当前核心页面只有 `/` 和 `/login`。
- 状态管理：Zustand + `persist`，主要用于认证状态持久化。
- 服务端状态：React Query，统一在 `frontend/src/lib/queryClient.ts` 定义默认策略。
- HTTP 层：Axios，底层 client 在 `frontend/src/shared/api/http-client.ts`，`frontend/src/services/api.ts` 仅保留兼容 facade。
- 测试层：`Vitest + React Testing Library + MSW`，测试支撑位于 `frontend/src/test/`。

## 前端目录职责
- `frontend/src/main.tsx`：挂载根节点并注入 QueryClientProvider。
- `frontend/src/App.tsx`：仅负责转交 `AppRouter`。
- `frontend/src/routes.tsx`：路由与页面级 lazy load。
- `frontend/src/shared/api/http-client.ts`：统一 Axios client 与 token 注入。
- `frontend/src/shared/api/queryKeys.ts`：React Query key 真源。
- `frontend/src/stores/authStore.ts`：认证 token / user 的持久化状态。
- `frontend/src/features/dashboard/hooks/useDashboardData.ts`：dashboard 查询聚合与统计逻辑。
- `frontend/src/features/dashboard/model/useDashboardPage.ts`：dashboard 页面装配层状态与 mutation 编排。
- `frontend/src/entities/room/api/roomApi.ts` / `frontend/src/entities/binding/api/bindingApi.ts`：领域 API 出口。
- `frontend/src/features/auth-login/api/authApi.ts`：登录/验证码流程 API 真源。
- `frontend/src/features/bind-room/`：bind-room feature 样板，包含 `api / model / ui / index`。
- `frontend/src/services/api.ts`：兼容 facade，不再承载 HTTP client 实现细节。
- `frontend/src/pages/DashboardPage.tsx`：核心页面容器。

## 前端维护性热点
- `DashboardPage.tsx` 当前不仅是页面，还承担：
  - 认证门禁
  - 查询结果转换
  - room/binding 组合映射
  - 多个 modal 状态编排
  - mutation 之后的 query invalidation
- `BindRoomModal.tsx` 是较重的交互式组件，包含多步流程、路径树查询、重试、绑定提交、错误处理。
- `services/api.ts` 把多个领域的 API 聚合在同一文件，后续可按 feature 或 domain 拆分。
- `useDashboardData.ts` 曾同时保留 `useBindingsQuery` 和 deprecated 的 `useRoomsQuery`，说明前端数据源迁移尚未完全收敛。

## 前端架构升级候选方向
- 将页面容器逻辑从 `DashboardPage.tsx` 进一步下沉到 feature hooks / view model。
- 将 API 封装按领域拆分，而不是继续集中在一个 `api.ts`。
- 清理 deprecated 的查询路径，统一 dashboard 的真实数据源。
- 把大 modal 的流程与视图拆分，降低单组件复杂度。
- 优先复用现有 `comic-modal` 这类复合组件基础设施，避免在升级过程中重复造 UI 抽象。

## 当前已落地的最小边界
- bind-room 已有独立 feature public API，`DashboardPage` 通过 feature 出口装配，而不是直接依赖旧组件路径。
- query key 已收敛，绑定成功后会正确失效 `rooms.flagged` 与 `bindings`，不再使用散落字符串。
- 登录页和认证弹窗已经切到 `features/auth-login`，`services/api.ts` 不再是认证逻辑真源。
- 首批行为测试已覆盖：
  - 登录页成功/失败登录状态
  - `useBindRoomModal` 的路径选择、绑定成功、错误提示与 query invalidation
  - `useDashboardPage` 的认证门禁、房间装配与 mutation 后失效
