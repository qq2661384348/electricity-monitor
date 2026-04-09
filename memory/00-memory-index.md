# Electricity Monitor memory 索引

## 使用规则

- `./memory` 只记录跨会话仍然成立的仓库事实、边界、长期风险和验证入口。
- 不记录短期调试信息、最近一次通过数、dirty worktree、一次性进度、机器私有环境或任何凭据原文。
- 开始任务时先读本文件，再按任务范围阅读相关 memory；交付前只更新真正被本次改动影响的长期事实。

## 文件地图

- `memory/01-repo-shape.md`：仓库结构、目录职责、AGENTS 拓扑与前后端真源入口。
- `memory/02-runtime-and-config.md`：运行时配置加载顺序、环境语义、关键环境变量与 fail-fast 约束。
- `memory/03-deploy-and-risk-memory.md`：发布链路、release 产物、服务器职责和部署相关长期风险。
- `memory/04-architecture-hotspots.md`：后端与前端仍需持续关注的结构性热点。
- `memory/05-frontend-architecture.md`：前端技术基线、Bun 工具链真源、目录职责和前端边界。
- `memory/06-maintainability-seams.md`：当前可复用的后端与前端维护性接缝。
- `memory/07-testing-and-quality-gates.md`：测试入口、CI 门禁、readiness / smoke 契约与常用验证路径。
- `memory/08-quality-and-security-risks.md`：已确认的质量风险、安全风险与供应链风险。

## 更新准则

- 新写入的内容必须能从代码、配置、CI、脚本或当前文档真源验证。
- 已经失效或明显阶段性的内容应直接删除，不保留“历史状态说明”。
- 涉及架构、发布、验证入口或协作边界变化时，同时更新对应的 `AGENTS.md` 与相关文档真源。
