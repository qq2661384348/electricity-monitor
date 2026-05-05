# 前端源码 AGENTS

## 作用范围

- 本文件补充根目录 `AGENTS.md` 与 `frontend/AGENTS.md`，只约束 `frontend/src/` 下的源码协作方式。

## 真源与入口

- 前端启动顺序固定为：`main.tsx` 挂载根节点并注入 `QueryClientProvider`，`App.tsx` 先恢复会话再转交 `AppRouter`，`routes.tsx` 负责页面级 lazy load 与路由装配。
- React Query 默认策略真源在 `lib/queryClient.ts`；query key 真源在 `shared/api/queryKeys.ts`。
- 真实 HTTP client 真源是 `shared/api/http-client.ts`；`services/api.ts` 只是兼容 facade，不应重新演化成新的 API 真源。
- 认证状态真源在 `stores/authStore.ts`，其中 access token 只保存在内存，`isSessionReady` 用于控制首屏会话恢复。
- 公开运行时配置真源是 `entities/public-config/api/publicConfigApi.ts` 与 `features/public-config/model/usePublicConfig.ts`，用于读取机器人 QQ、管理员 QQ、第三方验证码参数和 QQ 验证码长度。

## 首选接缝

- 页面文件优先做页面容器和布局装配，复杂交互流程下沉到 `features/`，单领域访问下沉到 `entities/`。
- 新的接口访问优先复用 `shared/api/http-client.ts`、现有 query key 和 feature / entity 公共出口。
- 如果某个页面需要多个 mutation、query invalidation 和权限门禁，优先抽到 feature model，而不是继续把 `pages/*.tsx` 做胖。
- 认证相关请求必须复用 `shared/api/http-client.ts` 的 `withCredentials`、单次 refresh 和 401 重放机制。
- 登录、公告、教程或验证码弹窗需要展示机器人 QQ、管理员 QQ 或验证码参数时，复用 `usePublicConfig`，不要在 UI 中硬编码运行时值。

## 边界与禁止项

- 除兼容 facade 外，不要在业务代码里继续从 `@/services/api` 取真实 API；优先从 feature、entity 或 `shared/api` 的真源导入。
- 不要在页面或组件里重复写 token 注入、401 处理或 Axios 实例化。
- 不要把 refresh token 暴露给前端业务代码，也不要重新往本地存储写 access token。
- 不要把全局 provider、路由装配或 shared query key 分散到多个页面文件。

## 最小验证

- 改动前端结构、导入边界或公共 API 后，在 `frontend/` 目录至少运行 `bun run test`、`bun run lint`、`bun run build:prod`。
- 涉及前端边界调整时，也在仓库根目录运行 `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`，确保没有重新引入 `@/services/api` 的禁用导入。

## 同步要求

- 修改前端启动骨架、HTTP client、会话模型或共享 query key 真源时，同时更新 `memory/long-term/semantic/frontend-architecture.md` 与 `memory/long-term/semantic/frontend-seams.md`；若改变浏览器会话模型，再同步 `memory/decisions/browser-session-uses-memory-access-token-and-cookie-refresh.md`。
