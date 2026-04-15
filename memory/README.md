# Electricity Monitor memory 目录索引

## 使用入口

- 开始任务时先读本文件，再按任务范围进入对应主题目录。
- `memory/` 默认只保存跨会话仍然成立的长期事实、边界、真源入口、长期风险和验证规则。
- 代码、配置、CI、脚本和当前文档真源优先于 memory；当真源变化时，应同步修正对应 memory，而不是保留失效说明。

## 如何阅读

- 需要判断 `memory/` 怎么写、短期事项能否进入仓库时，先读 `01-governance/01-memory-rules.md`。
- 需要理解仓库结构、目录职责或各级 `AGENTS.md` 拓扑时，读 `01-governance/02-repo-shape-and-agents.md`。
- 需要处理运行时配置、环境变量、鉴权会话或 CORS 时，读 `02-runtime/`。
- 需要理解前后端架构、维护接缝或热点时，读 `03-architecture/`。
- 需要处理部署、release、测试、CI 或 smoke 契约时，读 `04-delivery/`。
- 需要复核质量风险、安全风险和仓库外部控制面边界时，读 `05-risks/01-quality-and-security-risks.md`。

## 如何写入

- 长期记忆使用常规编号文件，例如 `01-*.md`、`02-*.md`。
- 短期记忆不单独建专区；只有确有必要时，才在对应主题目录下创建 `st-<slug>.md`。
- 短期文件头必须包含：`状态`、`来源`、`最后校验`、`失效条件`。
- 短期事项一旦稳定，要么并入对应长期文件，要么直接删除；不要把一次性进度、脏工作树状态或单次调试记录混进长期文件。
- 不记录机器私有环境、真实凭据、最近一次通过数、单次审计统计、一次性迁移进度或无复用价值的过程噪音。

## 目录地图

- `01-governance/01-memory-rules.md`：memory 的写作规则、长期/短期边界和更新原则。
- `01-governance/02-repo-shape-and-agents.md`：仓库结构、目录职责、真源入口和 AGENTS 拓扑。
- `02-runtime/01-config-and-environments.md`：运行时配置加载规则、环境语义、关键依赖和 fail-fast 约束。
- `02-runtime/02-auth-session-and-cors.md`：鉴权会话、cookie 契约、CORS 与管理员提升规则。
- `03-architecture/01-backend-seams.md`：后端可维护性接缝和模块化边界。
- `03-architecture/02-frontend-architecture.md`：前端技术基线、工作区真源和目录职责。
- `03-architecture/03-frontend-seams.md`：前端可复用接缝、页面/feature/entity 边界和会话模型。
- `03-architecture/04-hotspots.md`：当前仍需持续关注的前后端结构性热点。
- `04-delivery/01-deploy-and-release.md`：部署主线、release 产物、服务器职责与共享部署契约。
- `04-delivery/02-testing-and-quality-gates.md`：测试入口、CI 门禁、readiness / smoke 契约与本地执行约定。
- `05-risks/01-quality-and-security-risks.md`：长期质量风险、安全风险、供应链风险与仓库外部安全边界。

## 更新原则

- 任何新增内容都必须能从代码、配置、脚本、CI 或当前文档真源验证。
- 架构、运行时、测试门禁、部署或协作边界变化时，同时更新对应 `AGENTS.md` 与文档真源。
- 修改 `memory/` 的结构、职责或目录索引时，要同时更新本文件和 `01-governance/02-repo-shape-and-agents.md`。
