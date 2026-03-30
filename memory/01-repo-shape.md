# Electricity Monitor 仓库记忆：仓库形态与模块边界

## 仓库定位

- 仓库是一个前后端同仓的电费监控系统，不是 monorepo 工具链仓，也不是纯基础设施仓。
- 后端主程序是 Rust/Axum 服务，前端是 React + Vite 单页应用。
- 生产交付当前以 Docker 镜像和 GitHub Actions 手动打包发布为主线。
- 项目级工作约束要求：每次任务开始前先阅读 `./memory`，交付前同步更新 `./memory`。

## 关键目录

- `src/`: 后端主代码，采用分层结构。
- `src/AGENTS.md`: 后端本地协作说明，补充根级 AGENTS，对 `src/` 的入口、分层与验证负责。
- `src/modules/AGENTS.md`: 后端模块化迁移说明，补充 `src/modules/` 的边界与迁移规则。
- `frontend/`: 前端 SPA。
- `frontend/src/AGENTS.md`: 前端本地协作说明，补充根级 AGENTS，对 `frontend/src/` 的启动骨架、HTTP 真源与验证负责。
- `frontend/src/features/AGENTS.md`: 前端 feature 层协作说明，约束 `api / model / ui / index.ts` 形态。
- `frontend/src/entities/AGENTS.md`: 前端 entity 层协作说明，约束单域网关与公共出口。
- `config/`: TOML 配置，按 default/development/production 分层。
- `migrations/`: Diesel 数据库迁移。
- `.github/workflows/`: CI/CD 工作流，当前有手动发布工作流。
- `.github/workflows/ci.yml`: 当前 Pull Request / 手动质量门禁工作流，负责后端测试、前端质量检查与架构守护。
- `deploy/`: 部署真源目录，包含 Dockerfile / Dockerfile.dockerignore、release 包模板，以及本地 Docker 调试脚本。
- `tests/`: 集成测试入口；当前已按 `contracts/`、`runtime/`、`support/`、`infra/` 分层。
- `docs/`: 架构、部署、测试、迁移等文档。

## 后端分层记忆

- `src/bootstrap/`: 启动装配入口，承接配置初始化、日志、路由装配、运行时初始化和 shutdown。
- `src/config/`: 配置模型与加载逻辑。
- `src/domain/models/`: 领域模型，如用户、房间、绑定、电费历史。
- `src/domain/services/`: 业务服务，如房间同步、电费抓取、通知、限流、验证码。
- `src/infrastructure/external/`: 统一 `reqwest` 客户端构造与 HTTP 状态错误映射。
- `src/infrastructure/`: 数据库、Redis、外部 HTTP、电费接口、NapCat HTTP 机器人服务、缓存、仓储。
- `src/handlers/`: HTTP handler。
- `src/routes/`: API 路由编排。
- `src/middleware/`: JWT 鉴权与日志中间件。
- `src/modules/auth/`: 渐进式模块化样板，当前用于统一 `Actor` 身份模型与鉴权中间件边界。
- `src/modules/room/`: 当前后端热点域迁移样板，承接 room/path_tree 的 application 编排。
- `src/modules/room_sync/`: 当前后端热点域迁移样板，承接手动同步、同步状态、同步历史、房间路径查询编排。
- `src/state.rs`: 全局应用状态，持有 DB/Redis/限流器/缓存等共享资源。
- `scripts/check-architecture.ps1`: 当前架构守护脚本，校验前端导入边界、optimized 文件残留和 room handler 直接实例化 repository。

## 前端边界

- 前端使用 `createBrowserRouter`，核心页面为 `/` 和 `/login`。
- 前端真实 HTTP client 真源是 `frontend/src/shared/api/http-client.ts`，负责 `/api` 前缀、token 注入和 401 处理。
- `frontend/src/services/api.ts` 仅保留兼容 facade，不再是前端 API 真源。
- query key 真源在 `frontend/src/shared/api/queryKeys.ts`。
- `frontend/src/features/auth-login/` 已成为认证 API 的真实 feature 出口。
- `frontend/src/features/bind-room/` 已作为 feature 样板建立 public API，`entities/room` 与 `entities/binding` 承担领域 API 出口。
- `frontend/src/features/dashboard/model/useDashboardPage.ts` 已成为 dashboard 页面装配逻辑的主入口。
- 当前前端构建产物会复制到根目录 `static/`，由后端静态文件服务托管；`static/` 仅保留目录占位，不再跟踪构建产物文件。
- 前端基础运行骨架是：
  - `main.tsx` 注入 QueryClientProvider
  - `App.tsx` 仅转发到 Router
  - `routes.tsx` 负责页面级 lazy load

## 当前工程形态记忆

- 后端是单体服务，但运行时依赖 Redis。
- 数据库当前是 PostgreSQL 主路径，MySQL 是预留类型，不是当前主实现。
- 发布链路已从“本地构建上传服务器”转为“GitHub Actions 构建 release artifact -> 服务器 deploy.sh”。
- release smoke 与本地 readiness test 已通过 `deploy/smoke.targets` 收敛到同一份检查契约。
- 前端现已接入 `Vitest + Testing Library + MSW` 的最小行为测试基建。
- release artifact 当前会附带 `release-manifest.json`，服务器部署结果写到 `deploy-result.json`。
- release 包当前还会携带 `smoke.sh` 和 `secrets/.gitkeep`。
- 根目录不再直接存放部署脚本、Dockerfile 或 compose 文件，相关资产统一收敛到 `deploy/`。

## AGENTS 导航补充

- 根目录 `AGENTS.md` 仍是全仓默认协作约束真源。
- 根目录 `AGENTS.md` 现以共享规则、命令矩阵、验证矩阵和跨目录同步要求为主，不再堆放局部实现细节。
- 后端与前端核心目录现在额外提供子级 `AGENTS.md`，用于补充局部边界、反模式和最小验证。
- 局部目录的实现边界、反模式和最小验证应优先写入对应子级 `AGENTS.md`，避免根文件继续膨胀。
- 进入 `src/`、`src/modules/`、`frontend/src/`、`frontend/src/features/`、`frontend/src/entities/` 工作时，应同时遵守根级与对应子级 AGENTS。

## 第二轮扫描补充

- 当前仓库的主要改进方向已经从“补功能”转向“升级架构并增强可维护性”。
- 第二轮基线应重点关注：
  - 后端大文件与职责集中点
  - 平行实现/重复实现
  - 前端页面容器与 UI 组件的状态聚合问题
  - 文档与真实代码基线的漂移
