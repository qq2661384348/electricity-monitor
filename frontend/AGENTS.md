# 前端工作区 AGENTS

本文件补充根目录 `AGENTS.md`，只约束 `frontend/` 工作区级协作方式。

## 包管理与脚本真源

- `bun` 是 `frontend/` 的唯一包管理真源；不要重新引入 `pnpm` 或 `npm` 双轨维护。
- `bun.lock` 是唯一前端 lockfile；不要重新提交 `pnpm-lock.yaml` 或 `package-lock.json`。
- `frontend/package.json` 负责前端脚本和 Bun 版本声明。
- `frontend/scripts/copy-static.ts` 是把 `dist/` 复制到根目录 `static/` 的唯一脚本真源；不要再写回内联 `node -e` 复制逻辑。

## 常用命令

- 安装依赖：`bun install --frozen-lockfile`
- 启动开发：`bun run dev`
- 行为测试：`bun run test`
- 代码检查：`bun run lint`
- 生产构建：`bun run build:prod`
- 依赖审计：`bun audit`

## 工作区约束

- `build:prod` 会先构建 `frontend/dist/`，再复制到根目录 `static/`，供后端静态文件服务和 Docker 构建使用。
- `static/` 只保留目录占位，不重新纳入版本控制。
- 浏览器会话模型固定为“内存 access token + HTTPOnly refresh cookie”；不要把 access token 重新持久化到 localStorage、sessionStorage 或 IndexedDB。
- 真实跨请求鉴权依赖 `withCredentials` 和 `/api/auth/refresh`，不要重新引入 body 传递 refresh token 的旧协议。
- 修改前端工具链、lockfile、CI 或构建脚本时，同时更新 `frontend/README.md`、`docs/guides/TESTING.md`、`docs/guides/DOCKER_DEPLOYMENT.md`、`memory/05-frontend-architecture.md`、`memory/07-testing-and-quality-gates.md`。
- 源码层边界由 `frontend/src/AGENTS.md` 及其子级文件负责；不要把源码层实现规则重新堆回本文件。

## 最小验证

- 涉及前端依赖、脚本、测试或发布链路改动时，至少运行 `bun run test`、`bun run lint`、`bun run build:prod`。
- 涉及前端导入边界或兼容 facade 时，还应在仓库根目录运行 `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`。
