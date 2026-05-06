# 📚 文档索引

## 快速入口

- [项目总览](./README.md)
- [快速开始](./guides/QUICKSTART.md)
- [Docker 部署指南](./guides/DOCKER_DEPLOYMENT.md)
- [架构设计](./architecture/ARCHITECTURE.md)
- [API 参考](./api/API_REFERENCE.md)

## 文档分类

### 项目与架构

- [README.md](./README.md) - 项目介绍、技术栈、环境配置
- [ARCHITECTURE.md](./architecture/ARCHITECTURE.md) - 分层架构与设计决策
- [ADR_INDEX.md](./architecture/ADR_INDEX.md) - 当前架构升级决策索引
- [DUAL_IMPLEMENTATION_LEDGER.md](./architecture/DUAL_IMPLEMENTATION_LEDGER.md) - 双实现收敛台账
- [MAINTAINABILITY_METRICS.md](./architecture/MAINTAINABILITY_METRICS.md) - 当前维护性指标与质量门禁 owner
- [DEPRECATED_CODE_HISTORY.md](./architecture/DEPRECATED_CODE_HISTORY.md) - 历史代码与遗留背景

### 开发与测试

- [QUICKSTART.md](./guides/QUICKSTART.md) - 本地开发启动路径
- [BUILD_CONFIGURATION.md](./guides/BUILD_CONFIGURATION.md) - 编译配置与构建问题排查
- [DATABASE_MIGRATION.md](./guides/DATABASE_MIGRATION.md) - 迁移命令与数据库演进
- [TESTING.md](./guides/TESTING.md) - 当前测试真源、CI 门禁与 smoke/readiness 契约
- [TECHNICAL_DEBT.md](./guides/TECHNICAL_DEBT.md) - 技术债与后续重构切入点
- [SECRETS_INVENTORY.md](./guides/SECRETS_INVENTORY.md) - 生产 secrets 清单与轮换约定
- [RELEASE_SMOKE_CHECKLIST.md](./guides/RELEASE_SMOKE_CHECKLIST.md) - release 验收与回滚触发清单
- [ARCHITECTURE_REVIEW_CHECKLIST.md](./guides/ARCHITECTURE_REVIEW_CHECKLIST.md) - 架构升级后的 code review 审查清单
- [第三方验证码集成.md](./guides/第三方验证码集成.md) - 第三方验证码接入记录
- [NAPCAT_HTTP_SERVICE_GUIDE.md](./guides/NAPCAT_HTTP_SERVICE_GUIDE.md) - NapCat HTTP 机器人服务接入说明

### 部署与运维

- [DOCKER_DEPLOYMENT.md](./guides/DOCKER_DEPLOYMENT.md) - GitHub Actions artifact 发布链路

### API 文档

- [API_REFERENCE.md](./api/API_REFERENCE.md) - HTTP API 参考

## 当前真源约定

- 生产发布以 `.github/workflows/docker-build.yml` 和仓库 `deploy/` 目录为准。
- PR / 手动质量门禁以 `.github/workflows/ci.yml` 为准。
- release artifact 内的 `release-manifest.json` 是发布包身份真源，`deploy-result.json` 是服务器侧部署结果记录。
- release `.env` 是生产非敏感运行时值和 secret file 路径真源，包括 CORS、QQ 机器人发送地址、公开站点域名/端口、管理员 QQ、数据库/JWT/QQ/SMTP secret file 路径；生产配置模板不写入真实 secret 原文。
- release artifact 离线携带应用、PostgreSQL 和 Redis 镜像；服务器不从外部 registry 拉取这些镜像，默认通过 Docker Compose 管理 `postgres`、`redis`、一次性 `migrate` 和 `app`。
- out-of-repository deployment automation属于私有local environment文件，不纳入仓库；`.gitignore` 已忽略 `deploy/relay-deploy*.sh`。
- `deploy/smoke.targets` 是 readiness test 与 release smoke 共用的检查目标真源，包含端点、必需文件与统一响应安全头。
- `deploy/build.sh` 与 `deploy/docker-compose.local.yml` 仅用于本地 Docker 调试。
- 服务器上线消费 GitHub Actions 产出的 release artifact，不从源码重新构建。

## 文档维护

- 最后更新：2026-05-07
- 更新部署相关内容时，同时核对 `README.md`、`docs/guides/DOCKER_DEPLOYMENT.md`、`deploy/README.md` 与 `memory/long-term/procedural/deploy-and-release.md`
