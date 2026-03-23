# Electricity Monitor 仓库记忆：仓库形态与模块边界

## 仓库定位
- 仓库是一个前后端同仓的电费监控系统，不是 monorepo 工具链仓，也不是纯基础设施仓。
- 后端主程序是 Rust/Axum 服务，前端是 React + Vite 单页应用。
- 生产交付当前以 Docker 镜像和 GitHub Actions 手动打包发布为主线。

## 关键目录
- `src/`: 后端主代码，采用分层结构。
- `frontend/`: 前端 SPA。
- `config/`: TOML 配置，按 default/development/production 分层。
- `migrations/`: Diesel 数据库迁移。
- `.github/workflows/`: CI/CD 工作流，当前有手动发布工作流。
- `deploy/`: release 包模板文件，包括 compose、README、`.env.example`。
- `docs/`: 架构、部署、测试、迁移等文档。

## 后端分层记忆
- `src/config/`: 配置模型与加载逻辑。
- `src/domain/models/`: 领域模型，如用户、房间、绑定、电费历史。
- `src/domain/services/`: 业务服务，如房间同步、电费抓取、通知、限流、验证码。
- `src/infrastructure/`: 数据库、Redis、外部 HTTP、电费接口、QQ 机器人、缓存、仓储。
- `src/handlers/`: HTTP handler。
- `src/routes/`: API 路由编排。
- `src/middleware/`: JWT 鉴权与日志中间件。
- `src/state.rs`: 全局应用状态，持有 DB/Redis/限流器/缓存等共享资源。

## 前端边界
- 前端使用 `createBrowserRouter`，核心页面为 `/` 和 `/login`。
- API 通过 `frontend/src/services/api.ts` 访问 `/api` 前缀接口，依赖 Zustand 中的 token 注入。
- 当前前端构建产物会复制到根目录 `static/`，由后端静态文件服务托管。

## 当前工程形态记忆
- 后端是单体服务，但运行时依赖 Redis。
- 数据库当前是 PostgreSQL 主路径，MySQL 是预留类型，不是当前主实现。
- 发布链路已从“本地构建上传服务器”转为“GitHub Actions 构建 release artifact -> 服务器 deploy.sh”。
