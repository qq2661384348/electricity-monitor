# Handoff

## 当前状态

`add-email-module` 已完成 baseline、requirements、proposal、delta spec、design、traceability、tasks、acceptance、eval harness 和 consistency report。`consistency-report.md` 结论允许进入 apply。

## 执行顺序建议

1. 新增 email 配置域并接入 `AppConfig`。
2. 新增邮件基础设施模块。
3. 同步配置模板与 release secret 链路。
4. 同步 docs / memory 真源。
5. 运行验证并回写 tasks / worklog / scorecard。

## 关键依赖

- `lettre` async SMTP + rustls TLS。
- 现有 `src/config/app.rs` secret file 解析模式。
- 现有 `deploy/` Compose secrets 模式。

## 风险与注意事项

- SMTP 授权码不能写入 tracked 文件。
- 该 change 不改变 auth/API/frontend/database。
- 真实 SMTP 外部发送不纳入自动验收。

## 下一动作

进入 apply 并按 `tasks.md` 实施。

## Implementation Readiness

- 足够好的结果：后端新增可复用 email 模块，配置和 release secret 链路完整，自动测试覆盖配置/模板/校验/构建路径。
- 非目标：不接入 QQ 验证码替换、不新增 HTTP API、不改前端、不改数据库、不做真实 SMTP 外部发送。
- 待改文件 / 模块范围：`Cargo.toml`、`Cargo.lock`、`src/config/app.rs`、`src/config/mod.rs`、`src/config/email.rs`、`src/infrastructure/mod.rs`、`src/infrastructure/email/*`、`config/*.toml.example`、`deploy/release.env.example`、`deploy/compose.release.yml`、`deploy/deploy.sh`、相关 docs / memory / openspec 工件。
- 最小写集与必要性：配置文件用于新增运行时真源；基础设施文件用于 SMTP sender；deploy 文件用于生产 secret 注入；docs / memory 用于同步长期真源；openspec 文件用于本 change 闭环。
- 修改顺序：配置层 -> 邮件模块 -> 配置模板 -> release secret -> docs/memory -> 测试验证 -> 工件回写。
- 复用资产：`src/config` serde + Default 模式、`AppConfig::read_secret_file`、`src/infrastructure/notification` 模块组织、deploy Compose secrets 模式。
- 不允许触碰的边界：不修改 ignored `config/development.toml`；不修改 auth handler、verification service、frontend、数据库迁移、路由和现有 QQ 发送业务；不记录 SMTP 授权码原文。
- 预计新增或修改的测试：`src/config/email.rs` 单元测试、`src/config/app.rs` secret/production 校验测试、`src/infrastructure/email/sender.rs` 和 `templates.rs` 单元测试。
- 需要更新的文档 / 真源：`docs/guides/SECRETS_INVENTORY.md`、`docs/guides/DOCKER_DEPLOYMENT.md`、`docs/INDEX.md`、`deploy/README.md`、`deploy/README.release.md`、`README.md`、`docs/README.md`、`memory/long-term/semantic/config-and-environments.md`、`memory/long-term/procedural/deploy-and-release.md`、`memory/long-term/semantic/quality-and-security-risks.md`。
- 回滚 / fallback 思路：若 `lettre` feature 影响编译，优先调整同版本 rustls async feature；若 release secret 链路造成部署阻断，保持 email 配置缺省兼容，并确保 production 仅在显式配置 email SMTP 时要求 secret file。
- 无关重构 / 抽象 / 功能扩展确认：不做无关重构，不新增模板引擎、队列、限流、批量发送或业务接入。
- 是否存在仍会改变实现路径的 unknown：不存在。
- 是否允许进入 apply：允许。

## Source of Truth Sync

- 需要同步到 `openspec/specs/` 的 delta：`specs/email/spec.md` 已记录新增 email 配置、SMTP sender、验证码模板和 source-of-truth 同步要求。
- 需要同步到全局文档 / README / AGENTS / ./memory 的事实：新增 `[email]` 配置域、SMTP password secret file、release secret 挂载和部署校验。
- 需要长期保留的 ADR：`decisions/ADR-0001-email-smtp-uses-lettre-rustls.md`。
- 只作为 change-local 证据归档的内容：baseline、acceptance、eval harness、worklog、scorecard。
