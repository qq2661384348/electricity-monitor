# Architecture Review Checklist

用于本轮架构升级后的日常 code review。

## Backend

- Handler 不得直接承担复杂业务编排。
- Handler 不得新增直接实例化 repository 的代码，除非只是过渡兼容层并附带迁移说明。
- 新的请求身份相关逻辑应优先进入 `modules/auth`，不要把 admin token / JWT 差异重新扩散到 handler。
- 新能力优先进入 `bootstrap/` 或模块目录，不要回流到 `main.rs`。
- 不得新增新的双实现主线。

## Frontend

- 页面层应优先做装配，不直接承担大段流程状态和散落 query invalidation。
- 新 API 优先进入 `shared/api/http-client.ts`、`entities/*/api` 或 `features/*/api`，不要继续扩大 `services/api.ts`。
- 新 query key 必须进入 `shared/api/queryKeys.ts`。
- feature / entity 公开能力通过各自 `index.ts` 暴露。

## Release and docs

- 修改发布链时，同时同步 `README.md`、`docs/guides/DOCKER_DEPLOYMENT.md`、`docs/INDEX.md`、`deploy/README.md`、`memory/03-deploy-and-risk-memory.md`。
- 修改目录职责或边界时，同步 `memory/01-repo-shape.md` 与相关架构文档。
- 不在仓库里重新跟踪 `static/` 构建产物。
