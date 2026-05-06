# Electricity Monitor 前端说明

`frontend/` 是项目的 React 19 + Vite 8 前端工作区，负责登录页、首页仪表盘和房间绑定等浏览器端交互。

## 目录定位

- `src/main.tsx`：前端启动入口，负责挂载应用并注入全局 Provider。
- `src/App.tsx`：应用壳层，负责在首次渲染前通过 `/api/auth/refresh` 恢复会话，再转交路由。
- `src/routes.tsx`：页面级路由装配与 lazy load 真源。
- `src/shared/api/http-client.ts`：真实 HTTP client 真源，统一处理 `/api` 前缀、`withCredentials`、Bearer token 注入和单次 401 刷新重试。
- `src/stores/authStore.ts`：认证状态真源，只在内存里保存 access token，不做本地持久化。
- `src/entities/public-config/`：`/api/public-config` 的非敏感运行时配置 API 出口，供前端读取可用登录模式、机器人 QQ、管理员 QQ 和验证码参数。
- `src/features/public-config/`：公开配置 React Query hook，登录、公告、教程和验证码弹窗应复用这里的状态。
- `src/features/`：页面流程与交互编排层。
- `src/entities/`：单领域 API 与稳定公共出口。

## 包管理与常用命令

- `bun` 是 `frontend/` 的唯一包管理真源。
- `bun.lock` 是唯一前端 lockfile；不要重新引入 `pnpm-lock.yaml` 或 `package-lock.json`。
- `package.json` 中的 `overrides` 是前端传递依赖安全修复真源，用来把上游 advisory 锁定到已验证的修复版本。

在 `frontend/` 目录执行：

```bash
bun install --frozen-lockfile
bun run dev
bun run test
bun run lint
bun run build:prod
bun run check:bundle
bun audit
```

其中 `build:prod` 会先执行前端构建，再把 `dist/` 复制到仓库根目录 `static/`，供后端静态文件服务托管。`check:bundle` 会对 `dist/assets/*.js` 执行细粒度体积预算检查，预算真源如下：

- `lib-react-dom-* <= 192KB`
- `lib-framer-motion-* <= 144KB`
- `lib-react-router-* <= 96KB`
- 其他 JS chunk 一律 `<= 64KB`

## 当前约束

- 真实接口访问统一走 `src/shared/api/http-client.ts`，不要把 `src/services/api.ts` 再演化成新的接口真源。
- refresh token 只存在于 HTTPOnly Cookie；浏览器端只在内存保存 access token，页面刷新后的会话恢复统一走 `/api/auth/refresh`。
- 登录模式可用性、机器人 QQ、管理员 QQ、第三方验证码参数和登录验证码长度统一读取 `/api/public-config`；不要在页面或组件里硬编码这些运行时值。
- `vite.config.ts` 中的 `chunkSizeWarningLimit=192` 只是粗粒度 warning 阈值；真正的 chunk 预算约束以 `bun run check:bundle` 为准。
- 页面应尽量保持薄层，复杂交互优先下沉到 `src/features/`。
- 单领域访问和稳定公共出口优先收敛到 `src/entities/`。
- 根目录 `static/` 只保留构建产物占位，不重新纳入版本控制。

## 验证要求

前端结构、公共 API 或发布链路有变动时，至少运行以下命令：

```bash
bun run test
bun run lint
bun run build:prod
bun run check:bundle
bun audit
```

如涉及导入边界或兼容 facade，还应在仓库根目录补跑：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1
```

## 参考文档

- [前端局部 AGENTS](./src/AGENTS.md)
- [功能层 AGENTS](./src/features/AGENTS.md)
- [实体层 AGENTS](./src/entities/AGENTS.md)
- [根文档索引](../docs/INDEX.md)
