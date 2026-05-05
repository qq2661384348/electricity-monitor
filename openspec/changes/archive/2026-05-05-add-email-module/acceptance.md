# Acceptance

## 1. 验收目标

### 1.1 足够好的结果

- 后端新增可复用 email 基础设施模块。
- `[email]` 配置能被 `AppConfig` 加载，并支持 SMTP password secret file。
- 生产 release secret 链路完整。
- 单元测试和最相关质量门禁通过或有明确环境阻断说明。

### 1.2 非目标

- 不验证真实 SMTP 外部发送。
- 不改 auth/API/frontend/database。

## 2. 输入集合

### 2.1 场景类别

- 正常路径：有效配置、有效邮箱、普通邮件、验证码邮件。
- 边界路径：缺少配置、无效邮箱、无效验证码场景。
- 错误路径：production 配置了 email 但缺少 secret file。
- 回归路径：现有 config env 覆盖、secret file 覆盖、QQ/JWT/DB secret 校验仍有效。

### 2.2 具体样例集合

| 样例 | 输入 | 预期 | 断言 |
| --- | --- | --- | --- |
| S-001 | `[email] smtp_host=smtp.qq.com smtp_port=465 smtp_user=cogniaegis@qq.com` | 反序列化到 `AppConfig.email` | ACC-001 |
| S-002 | 收件人 `invalid-email` | 返回 validation error | ACC-003 |
| S-003 | `scene=login code=123456` | 主题/HTML/text 均包含验证码或场景语义 | ACC-004 |
| S-004 | production email 配置缺少 `smtp_password_file` | 配置校验失败 | ACC-006 |

## 3. 执行环境

### 3.1 运行入口

- Rust 单元测试：`cargo test --lib`
- release readiness 契约测试：`cargo test --test release_readiness_test`
- 格式化检查：`cargo fmt --check`
- Clippy：`cargo clippy --all-targets -- -D warnings`
- Rust 依赖审计：`cargo audit -q`
- 部署脚本语法：`bash -n deploy/deploy.sh`
- Docker Compose 配置自检：`docker compose -f deploy/docker-compose.local.yml config`

### 3.2 依赖服务

- 自动验收不依赖 PostgreSQL、Redis、SMTP 外部网络。

### 3.3 数据准备方式

- 测试使用临时 secret file 和内存样例。

### 3.4 环境变量 / 配置

- 不需要真实 SMTP password。
- 真实发送可在后续外部集成测试中设置 `APP__EMAIL__SMTP_PASSWORD_FILE`。

## 4. 执行方式

见 `eval/commands.sh` 与 `eval/runbook.md`。

## 5. 断言与检查项

### 5.1 功能正确性断言

- ACC-001：`AppConfig` 包含 email 配置，环境变量覆盖和 secret file 覆盖可用。
- ACC-002：`EmailSender::new` 对有效配置可构造 sender。
- ACC-003：无效收件人或发件人邮箱返回 validation/address 错误。
- ACC-004：验证码模板支持 register/login/reset/reset_password/bind/unbind。

### 5.2 数据正确性断言

- ACC-005：验证码模板输出包含验证码，不修改验证码内容，不把 password 写入输出。

### 5.3 边界 / 错误处理断言

- ACC-006：production 中配置 email SMTP 但缺少 `smtp_password_file` 时失败。
- ACC-007：部署 release 模板、compose secret 和 deploy 脚本 secret 校验引用一致。
- ACC-007b：release readiness 契约和 Rust 依赖审计通过。

### 5.4 回归断言

- ACC-008：不新增 auth/API/frontend/database 改动；现有配置测试仍通过。

## 6. 评分模型

| 维度 | 权重 |
| --- | --- |
| SC-functional 需求符合度与功能正确性 | 30 |
| SC-security secret 处理安全性 | 25 |
| SC-source-sync 配置/部署/文档真源同步 | 20 |
| SC-maintainability 最小性与可维护性 | 15 |
| SC-verification 验证覆盖度 | 10 |

## 7. 通过门槛

- 总分至少 85/100。
- SC-security 必须满分。
- ACC-001 到 ACC-008 必须全部满足，或对未执行项给出明确环境阻断且不影响自动路径。

## 8. 不通过条件

- tracked 文件包含 SMTP password 原文。
- production secret file 链路不一致。
- 邮件模块需要真实 SMTP 才能完成单元测试。
- 出现 auth/API/frontend/database 非目标改动。

## 9. 失败后的 repair 策略

- 配置失败：回到 `EmailConfig` / `AppConfig` 修正。
- 依赖或编译失败：优先调整 `lettre` feature，不引入 native TLS 平台依赖。
- 文档漂移：更新 traceability、source-of-truth 文档和 memory。

## 10. 产物与留痕要求

- 更新 `worklog.md` 记录 apply / verify 证据。
- verify 后生成 `scorecard.md`。
- `tasks.md` 只在任务完成后勾选。
