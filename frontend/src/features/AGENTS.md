# 前端功能层 AGENTS

## 作用范围

- 本文件补充根目录 `AGENTS.md`、`frontend/AGENTS.md` 与 `frontend/src/AGENTS.md`，只约束 `frontend/src/features/`。

## 真源与入口

- feature 默认按 `api / model / ui / index.ts` 组织；不是每个 feature 都必须四层齐全，但公共出口应收敛到 `index.ts`。
- `api/` 放 feature 级接口适配或流程专用调用。
- `model/` 放 view model、hook、query invalidation、流程状态和 mutation 编排。
- `ui/` 放 feature 私有展示组件；能复用到全局的基础组件应回到 `components/` 或共享 UI 资产。

## 边界与禁止项

- feature 可以协调 entity API、store、query invalidation 和页面流程，是页面容器和实体层之间的主装配层。
- `dashboard` 代表页面装配层接缝，`bind-room` 代表完整的 `api + model + ui` 样板，`auth-login` 代表认证流程 API 出口。
- 页面应尽量通过 feature 的公共出口消费能力，保持 `pages/` 薄而稳定。
- 不要把 route/provider 级启动逻辑塞进 feature。
- 不要在 feature 内重新造一套领域 API；已有 `entities/` 能承接的单域访问不要重复定义。
- 不要让 `ui/` 组件独自藏大量请求、副作用和 query invalidation；这类流程优先放回 `model/`。
- 不要绕过 `index.ts` 直接把内部实现路径当公共 API 长期扩散。

## 最小验证

- 变更 feature 公共出口、交互流程或 query invalidation 后，在 `frontend/` 目录至少运行 `bun run test`、`bun run lint`、`bun run build:prod`。
- 如果改动触及导入边界或兼容 facade，额外运行 `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`。
