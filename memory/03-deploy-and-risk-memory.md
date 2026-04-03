# Electricity Monitor 仓库记忆：部署链路与长期风险

## 当前生产发布主线
- GitHub Actions 工作流：`.github/workflows/docker-build.yml`
- 镜像构建真源：`deploy/Dockerfile`
- release 模板真源：`deploy/`
- 触发方式：手动 `workflow_dispatch`
- 输入：`git_tag`
- 输出：`release-<tag>.tar.gz`
- release 内容：
  - 应用镜像归档
  - Redis 镜像归档
  - `compose.yaml`
  - `deploy.sh`
  - `smoke.sh`
  - `smoke.targets`
  - `.env.example`
  - `README.md`
  - `release-manifest.json`

## 仓库内部署资产布局
- `deploy/Dockerfile` 与 `deploy/Dockerfile.dockerignore` 负责 GitHub Actions 镜像构建。
- `deploy/compose.release.yml`、`deploy/release.env.example`、`deploy/deploy.sh`、`deploy/smoke.sh`、`deploy/smoke.targets`、`deploy/README.release.md` 负责 release 包模板。
- `deploy/build.sh` 与 `deploy/docker-compose.local.yml` 只保留为本地 Docker 调试入口，不是生产发布真源。
- 根目录已不再直接放置部署相关文件，部署边界以 `deploy/` 目录为准。

## 服务器部署记忆
- 服务器不再从源码构建。
- 服务器职责：
  - `docker load`
  - 校验 `release-manifest.json`
  - 挂载 Compose secrets 指向的宿主机 secret files
  - `docker compose up`
  - 健康检查
  - 写出 `deploy-result.json`
  - 失败回滚
- 稳定容器名：
  - `electricity-app`
  - `electricity-redis`
- 默认健康检查：`/api/health`

## 构建性能记忆
- 前端依赖通过 pnpm lockfile 与缓存优化。
- Docker 镜像使用 Buildx + GHA cache。
- `deploy/Dockerfile` 继续复用 `cargo-chef` 多阶段构建。
- CI 直接构建 `linux/amd64` 镜像，避免开发机和服务器重复构建。
- GitHub Actions 在构建镜像前会先把 `config/production.toml.example` 复制为本地 `config/default.toml`，再把该运行时配置打进镜像。
- `deploy/Dockerfile` 现在会在构建期检查 `config/default.toml` 是否存在，缺失时直接 fail-fast。
- `deploy/build.sh` 在本地 Docker 调试前会自动补一份 `config/default.toml`（来源于 `config/development.toml.example`），避免因为运行时配置缺失导致镜像构建成功但启动失败。

## 长期风险记忆
- 仓库当前已切到“模板受版本控制、`config/default.toml` 本地化”的配置方式，但 git 历史中仍存在需单独处理的疑似敏感信息：
  - 已确认 `config/default.toml` 的 `qq_bot.bearer_token` 曾在 `9` 个历史提交中出现非占位符形态值
  - `origin/master` 已在 2026-04-03 完成历史重写并 force-push，`config/default.toml` 的历史已从仓库主线移除
  - 仓库与local environment环境已经完成去全局数据库密码环境变量收口，但真实 `QQ/JWT/DB` 凭据轮换仍需外部 secret file 或部署目标访问权
  - 在拿到外部访问能力前，不应把“已完成轮换”写入仓库记忆
- 文档主入口已基本收口，但后续仍需继续防止发布链路、模板配置和 memory 之间再次漂移。
- 仓库当前是 dirty worktree，后续做方案和交付时应注意区分已有用户修改与本次任务新增内容。
- 部署脚本的真实回滚路径还需要在 Linux Docker 主机上做一次端到端演练。

## 当前安全基线补充
- 鉴权主链路当前仍存在高优先级设计风险：
  - `/api/auth/refresh` 未区分 access token 与 refresh token
  - 前端 `auth-storage` 仍持久化 access token
  - 后端全局 CORS 仍为 `allow_origin(Any) + allow_methods(Any) + allow_headers(Any)`
  - `admin.default_qq_number` 在生产环境下尚未强制要求偏离默认值
- 这些问题都已进入最新的质量/安全报告，应作为后续修复任务的直接输入。

## 当前依赖与供应链风险补充
- 项目当前已经具备可执行的依赖审计路径：
  - `pnpm --dir frontend audit --prod`
  - `cargo audit`
- 最近一次基线检查结果：
  - 前端依赖命中 `7` 条 advisory，其中运行时直接依赖包含 `axios` 与 `react-router-dom`，构建链告警包含 `rollup` 与 `picomatch`
  - Rust 依赖命中 `5` 条漏洞与 `2` 条警告，涉及 `bytes`、`rkyv`、`rsa`、`rustls-webpki`、`time`、`bincode`、`lru`
- `frontend/package.json` 当前把 `@tailwindcss/vite` 放在 `dependencies`，会把构建链告警扩大到生产依赖审计面。

## 第二轮可维护性热点
- 后端存在多个超大文件：
  - `src/infrastructure/repositories/room_repository.rs` 约 1012 行
  - `src/domain/services/notification_gate.rs` 约 803 行
  - `src/domain/services/notification_service.rs` 约 636 行
  - `src/main.rs` 约 426 行
- 当前仍需继续关注的平行/过渡态实现：
  - 通知域和仓储边界仍有集中点，但 `room_sync/sync_service_optimized.rs` 与 `electricity_service_optimized.rs` 已删除
- 前端也存在职责集中：
  - `frontend/src/pages/DashboardPage.tsx` 同时承担容器状态、查询协调、模态编排
  - `frontend/src/components/BindRoomModal.tsx` 体量较大
  - `frontend/src/services/api.ts` 继续作为集中式 API 封装入口
- 这些热点更适合作为下一轮“前后端架构升级”和“维护性重构”的直接切入点。

## 第三轮补充
- 当前生产发布主线虽然已迁移到 GitHub Actions artifact，但真正的高可维护性升级不应只停留在部署链路，还需要同步处理：
  - 鉴权模型边界
  - 缓存架构是否真正落地
  - 通知域拆分
  - 前端页面容器与复用组件的关系

## 当前补齐进展
- release artifact 已开始携带 `release-manifest.json`，包含 tag、git SHA、镜像 digest 与归档校验值。
- `deploy.sh` 会读取 manifest 做基础一致性校验，并在 release 目录落 `deploy-result.json` 作为部署结果记录。
- release 包已提供 `smoke.sh`，用于部署后验证 `/api/health`、`/api/health/db`、静态入口与 manifest/result 文件。
- `release_readiness_test.rs` 与 `deploy/smoke.sh` 现通过 `deploy/smoke.targets` 共享 health / static / artifact 检查目标，避免双轨硬编码漂移。
- `.github/workflows/ci.yml` 现已补齐 frontend-tests job，并统一上传 backend/frontend/architecture 日志 artifact 供失败定位。
- release 包的 `.env.example` 现在只暴露 `*_SECRET_FILE` 路径，不再要求把秘密原文写进 `.env`。
- `electricity_service_optimized.rs` 已移除，电费写入主线明确为 `electricity_service.rs`。
- `room_sync/sync_service_optimized.rs` 已移除，房间同步主线明确为 `room_sync/sync_service.rs`。
- 若协作者仍持有历史重写前的 clone，应重新 clone 或按重写后主线彻底清理旧历史，避免把旧提交重新推回远端。
