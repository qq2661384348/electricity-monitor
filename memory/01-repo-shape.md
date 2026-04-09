# Electricity Monitor 仓库记忆：仓库形态与模块边界

## 仓库定位

- 仓库是一个前后端同仓的电费监控系统；后端是 Rust + Axum 单体服务，前端是 React + Vite 单页应用。
- 生产交付主线是 GitHub Actions 构建 release artifact，再由服务器执行 release 包内脚本完成部署。
- `memory/00-memory-index.md` 是 memory 入口；协作规则以根级与就近子级 `AGENTS.md` 为准。

## 关键目录

- `src/`：后端主代码，启动装配在 `src/bootstrap/`，共享运行时资源在 `src/state.rs`。
- `src/modules/`：后端模块化迁移主线，复杂编排优先在模块的 `application/` 落地。
- `frontend/`：前端工作区；`bun` 是唯一包管理真源，`bun.lock` 是唯一前端 lockfile。
- `frontend/src/`：前端源码；真实 HTTP client 真源在 `frontend/src/shared/api/http-client.ts`。
- `config/`：运行时配置模板与本地 `default.toml` 入口。
- `deploy/`：部署真源目录，包含 Dockerfile、release 模板和本地 Docker 调试脚本。
- `tests/`：后端契约、runtime、support 与 infra 分层。
- `docs/`：架构、测试、部署和运维文档。
- `.github/workflows/`：CI 与 release 工作流。

## 前后端真源入口

- 后端路由编排真源在 `src/routes/`，复杂业务编排应优先下沉到 `src/modules/*/application`。
- 前端启动骨架固定为 `frontend/src/main.tsx` -> `frontend/src/App.tsx` -> `frontend/src/routes.tsx`。
- 前端 API 真源是 `frontend/src/shared/api/http-client.ts`；`frontend/src/services/api.ts` 只保留兼容 facade。
- 前端构建真源是 `frontend/package.json` 的 `build:prod` 脚本；它会在构建后把 `frontend/dist/` 复制到根目录 `static/`。
- readiness test 与 release smoke 共用 `deploy/smoke.targets` 作为检查目标真源。

## 工程形态

- 后端运行时依赖 PostgreSQL 与 Redis。
- 前端使用 React Query、Zustand、Vitest、Testing Library 和 MSW。
- 根目录 `static/` 只保留占位文件；不跟踪前端构建产物。
- 根目录不再直接存放部署脚本或 Docker 编排文件，部署资产统一收敛在 `deploy/`。

## AGENTS 拓扑

- 根目录 `AGENTS.md` 定义全仓共享规则、命令真源与同步要求。
- `src/AGENTS.md` 约束后端入口、共享状态与最小验证。
- `src/modules/AGENTS.md` 约束模块化边界和 `api / application / domain / infrastructure` 分层。
- `frontend/AGENTS.md` 约束前端工作区级包管理、构建、测试与静态产物规则。
- `frontend/src/AGENTS.md`、`frontend/src/features/AGENTS.md`、`frontend/src/entities/AGENTS.md` 约束前端源码层边界。
- 进入更深目录工作时，应同时遵守根级与就近子级 `AGENTS.md`。
