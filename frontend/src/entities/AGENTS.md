# 前端实体层 AGENTS

## 作用范围

- 本文件补充根目录 `AGENTS.md`、`frontend/AGENTS.md` 与 `frontend/src/AGENTS.md`，只约束 `frontend/src/entities/`。

## 真源与入口

- `entities/` 是前端按领域收敛的单域网关层，当前以 `room`、`binding` 和 `public-config` 为主。
- 实体层应该保持 UI 无关，主要承接单域 API、必要的领域转换和稳定公共出口。
- 每个实体目录都应通过 `index.ts` 暴露公共 API，避免业务层长期直接依赖深层文件路径。
- HTTP 访问统一复用 `shared/api/http-client.ts`。
- query key 统一复用 `shared/api/queryKeys.ts`，不要在实体层散落重复字符串 key。
- `entities/public-config/` 只承接 `/api/public-config` 的非敏感运行时配置，供页面层和 feature 层读取机器人 QQ、管理员 QQ、验证码参数和 QQ 验证码长度。

## 边界与禁止项

- 实体层可以暴露单域请求函数或轻量领域工具，但跨实体编排、页面流程和 query invalidation 应优先留在 feature model。
- 不要在实体层放 modal/page state、页面事件处理或路由跳转逻辑。
- 不要在实体层里做跨多个实体的组合编排；那是 feature 的职责。
- 不要重新引入对 `@/services/api` 的依赖，或在实体层里直接操作全局页面组件。

## 最小验证

- 变更实体公共出口或 API 封装后，至少确认对应 `index.ts`、`api/*` 与 `shared/api/http-client.ts` 的引用仍一致。
- 涉及实体层边界调整时，在 `frontend/` 目录运行 `bun run test`、`bun run lint`、`bun run build:prod`，并用 `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1` 检查禁用导入是否回归。
