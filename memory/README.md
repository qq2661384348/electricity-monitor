---
type: index
status: active
scope: 全项目
created_at: 2026-04-17
updated_at: 2026-05-06
summary: Electricity Monitor 项目记忆系统总入口
---

# Electricity Monitor 项目记忆索引

## 记忆结构

- `long-term/semantic/`：长期稳定事实、模块边界、配置真源、架构热点和长期风险。
- `long-term/procedural/`：长期稳定流程、治理规则、部署契约和质量门禁。
- `short-term/working/`：短期工作态目录；固定入口是 `short-term/working/current.md`。
- `issues/`：结构化问题记录；只有值得长期复用的排障资产才进入。
- `decisions/`：影响协作边界、运行时契约或长期维护方式的稳定决策。

## 当前优先入口

- 当前工作态：`short-term/working/current.md`
- 关键长期事实：
  - `long-term/semantic/repo-shape-and-agents.md`
  - `long-term/semantic/config-and-environments.md`
  - `long-term/semantic/auth-session-and-cors.md`
  - `long-term/semantic/frontend-architecture.md`
  - `long-term/semantic/backend-seams.md`
  - `long-term/semantic/quality-and-security-risks.md`
- 关键长期流程：
  - `long-term/procedural/memory-governance.md`
  - `long-term/procedural/deploy-and-release.md`
  - `long-term/procedural/testing-and-quality-gates.md`
- 关键决策：
  - `decisions/frontend-package-manager-is-bun.md`
  - `decisions/runtime-config-uses-environment-named-toml.md`
  - `decisions/browser-session-uses-memory-access-token-and-cookie-refresh.md`

## 当前短期记忆状态

- 当前没有活动中的短期工作态。
- `short-term/working/current.md` 是固定入口；只有在出现需要跨会话延续、但尚未稳定到长期层的工作态时才填充内容或拆分额外短期文件。
- 额外短期文件名统一使用 `st-<slug>.md`；创建时必须显式写明状态、来源、最后校验和失效条件。

## 最近重要变化

- 2026-05-06：登录与通知链路扩展为 QQ / 邮箱双模式；账号数据按 `login_provider` 隔离，邮箱登录本轮固定普通用户，邮箱通知复用 SMTP 邮件模块。
- 2026-04-17：`memory/` 从自定义主题编号结构切换到 `project-memory` 默认主干。
- 2026-04-17：全局索引入口统一为 `memory/README.md`，短期工作态固定入口统一为 `short-term/working/current.md`。
- 2026-04-17：Bun 包管理、环境命名配置文件和浏览器会话模型三项协作边界被单独提升为决策记录。

## 维护约定

- 长期记忆只收录稳定、可复用、可验证的知识，不记录一次性进度、脏工作树状态或单次调试噪音。
- 短期记忆只作为例外存在；稳定后要并入长期层，失效后要删除或归档。
- 重要问题不强行补历史档案；只有真正形成排障资产时才进入 `issues/`。
- 重要协作边界、运行时契约或长期策略变化，应在 `decisions/` 留痕，并同步修正相关长期文件。
