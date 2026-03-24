# Electricity Monitor 仓库记忆：部署链路与长期风险

## 当前生产发布主线
- GitHub Actions 工作流：`.github/workflows/docker-build.yml`
- 触发方式：手动 `workflow_dispatch`
- 输入：`git_tag`
- 输出：`release-<tag>.tar.gz`
- release 内容：
  - 应用镜像归档
  - Redis 镜像归档
  - `compose.yaml`
  - `deploy.sh`
  - `.env.example`
  - `README.md`

## 服务器部署记忆
- 服务器不再从源码构建。
- 服务器职责：
  - `docker load`
  - `docker compose up`
  - 健康检查
  - 失败回滚
- 稳定容器名：
  - `electricity-app`
  - `electricity-redis`
- 默认健康检查：`/api/health`

## 构建性能记忆
- 前端依赖通过 pnpm lockfile 与缓存优化。
- Docker 镜像使用 Buildx + GHA cache。
- Dockerfile 继续复用 `cargo-chef` 多阶段构建。
- CI 直接构建 `linux/amd64` 镜像，避免开发机和服务器重复构建。

## 长期风险记忆
- 配置文件中仍存在明文敏感信息：
  - 数据库账号/密码
  - JWT 默认 secret / admin token
  - QQ 机器人 bearer token
- 文档存在一定漂移：
  - `docs/INDEX.md` 仍有“待创建”描述，未完全反映当前部署文档状态。
- 仓库当前是 dirty worktree，后续做方案和交付时应注意区分已有用户修改与本次任务新增内容。
- 部署脚本的真实回滚路径还需要在 Linux Docker 主机上做一次端到端演练。

## 第二轮可维护性热点
- 后端存在多个超大文件：
  - `src/infrastructure/repositories/room_repository.rs` 约 1012 行
  - `src/domain/services/notification_gate.rs` 约 803 行
  - `src/domain/services/notification_service.rs` 约 636 行
  - `src/main.rs` 约 426 行
- 存在平行实现/重复实现：
  - `electricity_service.rs` 与 `electricity_service_optimized.rs`
  - `room_sync/sync_service.rs` 与 `room_sync/sync_service_optimized.rs`
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
