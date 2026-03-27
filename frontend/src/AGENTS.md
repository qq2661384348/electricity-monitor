# Frontend Local AGENTS

本文件补充根目录 `AGENTS.md`，只约束 `frontend/src/` 下的前端协作方式。

## 启动骨架与真源

- 前端启动顺序固定为：`main.tsx` 挂载根节点并注入 `QueryClientProvider`，`App.tsx` 只转交 `AppRouter`，`routes.tsx` 负责页面级 lazy load 与路由装配。
- React Query 默认策略真源在 `lib/queryClient.ts`；query key 真源在 `shared/api/queryKeys.ts`。
- 真实 HTTP client 真源是 `shared/api/http-client.ts`；`services/api.ts` 只是兼容 facade，不应重新演化成新的 API 大杂烩。
- 认证持久化状态真源在 `stores/authStore.ts`。

## 首选接缝

- 页面文件优先做页面容器和布局装配，复杂交互流程下沉到 `features/`，领域 API 下沉到 `entities/`。
- 新的接口访问优先复用 `shared/api/http-client.ts`、现有 query key 和 feature/entity 公共出口。
- 如果某个页面需要多个 mutation、query invalidation 和权限门禁，优先抽到 feature model，而不是继续把 `pages/*.tsx` 做胖。
- 生产构建仍以 `pnpm --dir frontend build:prod` 为真源，并由发布链路复制产物到根目录 `static/` 后交给后端托管。

## 明确禁止

- 除兼容 facade 外，不要在业务代码里继续从 `@/services/api` 取真实 API；优先从 feature、entity 或 `shared/api` 的真源导入。
- 不要在页面或组件里重复写 token 注入、401 处理或 Axios 实例化。
- 不要把全局 provider、路由装配或 shared query key 分散到多个页面文件。

## 最小验证

- 改动前端结构、导入边界或公共 API 后，至少运行 `pnpm --dir frontend test`、`pnpm --dir frontend lint`、`pnpm --dir frontend build:prod`。
- 涉及前端边界调整时，也运行 `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`，确保没有重新引入 `@/services/api` 的禁用导入。
