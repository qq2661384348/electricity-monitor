# 前端工作区 AGENTS

## 作用范围

- 本文件补充根目录 `AGENTS.md`，只约束 `frontend/` 工作区级协作方式。

## 包管理与脚本真源

- `bun` 是 `frontend/` 的唯一包管理真源；不要重新引入 `pnpm` 或 `npm` 双轨维护。
- `bun.lock` 是唯一前端 lockfile；不要重新提交 `pnpm-lock.yaml` 或 `package-lock.json`。
- `frontend/package.json` 负责前端脚本和 Bun 版本声明。
- `frontend/package.json` 中的 `overrides` 是前端传递依赖安全修复真源；升级工具链后要让 Bun 锁文件和 overrides 保持一致。
- `frontend/scripts/copy-static.ts` 是把 `dist/` 复制到根目录 `static/` 的唯一脚本真源；不要再写回内联 `node -e` 复制逻辑。
- `frontend/scripts/check-bundle-budgets.ts` 是前端 JS chunk 预算真源；不要把体积约束重新散落到 CI 或文档里。

## 工作区约束

- `build:prod` 会先构建 `frontend/dist/`，再复制到根目录 `static/`，供后端静态文件服务和 Docker 构建使用。
- `vite.config.ts` 中的 `chunkSizeWarningLimit` 只负责粗粒度 warning；真正的 chunk 上限由 `check:bundle` 维护。
- `static/` 只保留目录占位，不重新纳入版本控制。
- 浏览器会话模型固定为“内存 access token + HTTPOnly refresh cookie”；不要把 access token 重新持久化到 `localStorage`、`sessionStorage` 或 `IndexedDB`。
- 真实跨请求鉴权依赖 `withCredentials` 和 `/api/auth/refresh`，不要重新引入 body 传递 refresh token 的旧协议。
- 源码层边界由 `frontend/src/AGENTS.md` 及其子级文件负责；不要把源码层实现规则重新堆回本文件。

## 最小验证

- 安装依赖：`bun install --frozen-lockfile`
- 启动开发：`bun run dev`
- 行为测试：`bun run test`
- 代码检查：`bun run lint`
- 生产构建：`bun run build:prod`
- Bundle 预算检查：`bun run check:bundle`
- 依赖审计：`bun audit`
- 涉及前端导入边界或兼容 facade 时，还应在仓库根目录运行 `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`。

## 同步要求

- 修改前端工具链、lockfile、CI 或构建脚本时，同时更新 `frontend/README.md`、`docs/guides/TESTING.md`、`docs/guides/DOCKER_DEPLOYMENT.md`、`memory/03-architecture/02-frontend-architecture.md`、`memory/03-architecture/03-frontend-seams.md`、`memory/04-delivery/02-testing-and-quality-gates.md`。
