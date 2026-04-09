# Electricity Monitor 仓库记忆：前端架构专项

## 技术基线

- 框架：React 19 + TypeScript + Vite 7。
- 包管理：`bun` 是 `frontend/` 的唯一包管理真源，`bun.lock` 是唯一前端 lockfile。
- 路由：`createBrowserRouter`，核心页面是 `/` 和 `/login`。
- 状态管理：Zustand + `persist`，主要承载认证状态。
- 服务端状态：React Query，默认策略在 `frontend/src/lib/queryClient.ts`。
- HTTP 层：Axios；真实 client 在 `frontend/src/shared/api/http-client.ts`，`frontend/src/services/api.ts` 仅保留兼容 facade。
- 测试层：Vitest + React Testing Library + MSW，测试支撑位于 `frontend/src/test/`。

## 工作区真源

- `frontend/package.json` 负责前端脚本与 Bun 版本声明。
- `frontend/scripts/copy-static.ts` 负责把 `frontend/dist/` 复制到根目录 `static/`。
- `frontend/AGENTS.md` 约束前端工作区级包管理、构建与验证；`frontend/src/**/AGENTS.md` 负责源码层边界。

## 前端目录职责

- `frontend/src/main.tsx`：挂载根节点并注入 QueryClientProvider。
- `frontend/src/App.tsx`：仅负责转交 `AppRouter`。
- `frontend/src/routes.tsx`：路由与页面级 lazy load 真源。
- `frontend/src/shared/api/http-client.ts`：统一 `/api` 前缀、token 注入和 401 处理。
- `frontend/src/shared/api/queryKeys.ts`：React Query key 真源。
- `frontend/src/stores/authStore.ts`：认证 token / user 的持久化状态。
- `frontend/src/features/auth-login/api/authApi.ts`：登录与验证码流程 API 真源。
- `frontend/src/features/bind-room/`：bind-room feature 样板，包含 `api / model / ui / index`。
- `frontend/src/entities/room/api/roomApi.ts` 与 `frontend/src/entities/binding/api/bindingApi.ts`：单领域 API 出口。
- `frontend/src/features/dashboard/model/useDashboardPage.ts`：dashboard 页面装配层状态与 mutation 编排。

## 验证入口

- 在 `frontend/` 目录执行 `bun install --frozen-lockfile` 安装依赖。
- 在 `frontend/` 目录执行 `bun run test`、`bun run lint`、`bun run build:prod` 做前端最小回归。
- 在 `frontend/` 目录执行 `bun audit` 做前端依赖审计。

## 仍需持续关注的前端热点

- `DashboardPage.tsx` 仍是高耦合页面容器。
- `BindRoomModal.tsx` 仍是多步骤流程型组件。
- `frontend/src/services/api.ts` 仍是需要继续缩小职责的兼容 facade。
- `frontend/src/features/dashboard/hooks/useDashboardData.ts` 仍保留迁移痕迹，说明 dashboard 数据层尚未完全收敛。
