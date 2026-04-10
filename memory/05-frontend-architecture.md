# Electricity Monitor 仓库记忆：前端架构专项

## 技术基线

- 框架：React 19 + TypeScript + Vite 7。
- 包管理：`bun` 是 `frontend/` 的唯一包管理真源，`bun.lock` 是唯一前端 lockfile。
- 路由：`createBrowserRouter`，核心页面是 `/` 和 `/login`。
- 状态管理：Zustand，认证状态只保存在内存中，不再做本地持久化。
- 服务端状态：React Query，默认策略在 `frontend/src/lib/queryClient.ts`。
- HTTP 层：Axios；真实 client 在 `frontend/src/shared/api/http-client.ts`，默认开启 `withCredentials`，`frontend/src/services/api.ts` 仅保留兼容 facade。
- 测试层：Vitest + React Testing Library + MSW，测试支撑位于 `frontend/src/test/`。

## 工作区真源

- `frontend/package.json` 负责前端脚本与 Bun 版本声明。
- `frontend/scripts/copy-static.ts` 负责把 `frontend/dist/` 复制到根目录 `static/`。
- `frontend/AGENTS.md` 约束前端工作区级包管理、构建与验证；`frontend/src/**/AGENTS.md` 负责源码层边界。

## 前端目录职责

- `frontend/src/main.tsx`：挂载根节点并注入 QueryClientProvider。
- `frontend/src/App.tsx`：在首次渲染前调用 `/api/auth/refresh` 恢复会话，再转交 `AppRouter`。
- `frontend/src/routes.tsx`：路由与页面级 lazy load 真源。
- `frontend/src/shared/api/http-client.ts`：统一 `/api` 前缀、Bearer token 注入、`withCredentials`、单次 refresh 和 401 重放。
- `frontend/src/shared/api/queryKeys.ts`：React Query key 真源。
- `frontend/src/stores/authStore.ts`：认证 token / user 的内存态状态，额外维护 `isSessionReady`。
- `frontend/src/features/auth-login/api/authApi.ts`：登录与验证码流程 API 真源。
- `frontend/src/features/bind-room/`：bind-room feature 样板，包含 `api / model / ui / index`。
- `frontend/src/entities/room/api/roomApi.ts` 与 `frontend/src/entities/binding/api/bindingApi.ts`：单领域 API 出口。
- `frontend/src/features/dashboard/model/useDashboardPage.ts`：dashboard 页面装配层状态与 mutation 编排。

## 验证入口

- 在 `frontend/` 目录执行 `bun install --frozen-lockfile` 安装依赖。
- 在 `frontend/` 目录执行 `bun run test`、`bun run lint`、`bun run build:prod` 做前端最小回归。
- 在 `frontend/` 目录执行 `bun audit` 做前端依赖审计。

## 会话模型

- 浏览器端只在内存持有 access token。
- refresh token 只存在于后端签发的 HTTPOnly Cookie，不通过 JSON 暴露给前端代码。
- 页面刷新后的会话恢复统一走 `/api/auth/refresh`。
- 并发 401 回收敛到单个 in-flight refresh promise，避免同一轮失效触发刷新风暴。

## 仍需持续关注的前端热点

- `DashboardPage.tsx` 仍是高耦合页面容器。
- `BindRoomModal.tsx` 仍是多步骤流程型组件。
- `frontend/src/services/api.ts` 仍是需要继续缩小职责的兼容 facade。
- `frontend/src/features/dashboard/hooks/useDashboardData.ts` 仍保留迁移痕迹，说明 dashboard 数据层尚未完全收敛。
