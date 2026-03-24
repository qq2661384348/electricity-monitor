# Electricity Monitor 仓库记忆：前端架构专项

## 当前前端技术基线
- 框架：React 19 + TypeScript + Vite 7。
- 路由：`createBrowserRouter`，当前核心页面只有 `/` 和 `/login`。
- 状态管理：Zustand + `persist`，主要用于认证状态持久化。
- 服务端状态：React Query，统一在 `frontend/src/lib/queryClient.ts` 定义默认策略。
- HTTP 层：Axios，集中封装在 `frontend/src/services/api.ts`。

## 前端目录职责
- `frontend/src/main.tsx`：挂载根节点并注入 QueryClientProvider。
- `frontend/src/App.tsx`：仅负责转交 `AppRouter`。
- `frontend/src/routes.tsx`：路由与页面级 lazy load。
- `frontend/src/stores/authStore.ts`：认证 token / user 的持久化状态。
- `frontend/src/features/dashboard/hooks/useDashboardData.ts`：dashboard 查询聚合与统计逻辑。
- `frontend/src/services/api.ts`：认证、房间、绑定等 API 访问封装。
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
- `useDashboardData.ts` 中同时保留了 `useBindingsQuery` 和 deprecated 的 `useRoomsQuery`，说明前端数据源迁移尚未完全收敛。

## 前端架构升级候选方向
- 将页面容器逻辑从 `DashboardPage.tsx` 进一步下沉到 feature hooks / view model。
- 将 API 封装按领域拆分，而不是继续集中在一个 `api.ts`。
- 清理 deprecated 的查询路径，统一 dashboard 的真实数据源。
- 把大 modal 的流程与视图拆分，降低单组件复杂度。
- 优先复用现有 `comic-modal` 这类复合组件基础设施，避免在升级过程中重复造 UI 抽象。
