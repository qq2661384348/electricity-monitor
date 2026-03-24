# ADR Index

本文件记录当前架构升级过程中已经落地或待确认的关键决策索引。

## 已落地

| ADR | 主题 | 当前状态 | 证据 |
| --- | --- | --- | --- |
| ADR-001 | 后端入口薄化到 `bootstrap::app::run()` | Accepted | `src/main.rs`, `src/bootstrap/` |
| ADR-002 | 请求身份统一为 `Actor`，旧 `UserContext` 保留兼容投影 | Accepted | `src/modules/auth/`, `src/middleware/auth.rs` |
| ADR-003 | 前端 HTTP client / query key / bind-room feature 收敛 | Accepted | `frontend/src/shared/api/`, `frontend/src/features/bind-room/` |
| ADR-004 | release artifact 增加 `release-manifest.json`，服务器侧写 `deploy-result.json` | Accepted | `.github/workflows/docker-build.yml`, `deploy/deploy.sh` |
| ADR-005 | `static/` 不再跟踪构建产物，只保留目录占位 | Accepted | `.gitignore`, `static/.gitkeep`, `frontend/package.json` |

## 待确认

| ADR | 主题 | 当前状态 | 待确认点 |
| --- | --- | --- | --- |
| ADR-006 | 生产 secrets 注入主线 | Accepted | 使用 Compose secrets |
| ADR-007 | Redis 长期部署模型 | Accepted | 与应用共部署到同一台设备 |
| ADR-008 | 管理员固定 token 策略 | Accepted | 移除固定 token，管理员改走验证码登录 + JWT |
| ADR-009 | `CacheManager` 命运 | Accepted | 正式接入主链路 |
| ADR-010 | 双实现收敛顺序 | Accepted | 先收敛 `electricity_service_optimized` |
| ADR-011 | 前端是否中期独立发布 | Accepted | 继续由后端托管静态资源 |
| ADR-012 | worker 是否需要独立部署 | Accepted | 当前不推进，继续保持单体进程 + Redis 拓扑 |
| ADR-013 | 是否立即接入统一观测平台 | Accepted | 当前不推进 OTel collector / metrics backend，先维持 tracing + smoke |
| ADR-014 | 是否立即引入更细粒度权限模型 | Accepted | 当前不推进，先收敛 JWT + role 边界 |
