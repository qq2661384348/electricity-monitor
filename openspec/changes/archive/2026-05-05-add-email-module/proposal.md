# Proposal

## 目标

新增一个可复用后端邮件模块，提供 SMTP 发送、验证码邮件模板和配置真源支持，并把生产 SMTP 授权码纳入现有 secret file / release 部署链路。

## 范围

- 新增 `src/config/email.rs` 并聚合到 `AppConfig`。
- 新增 `src/infrastructure/email/`，包含错误类型、sender 和验证码模板。
- 新增 `lettre` 依赖，使用 Tokio async + rustls TLS。
- 更新配置模板、release secret 挂载和部署校验。
- 更新配置/部署/secrets 文档与 memory。
- 新增/更新后端单元测试。

## 非目标

- 不接入现有 QQ 验证码登录。
- 不新增邮箱登录、注册、绑定、重置密码或邮件通知业务 API。
- 不新增数据库 schema。
- 不做真实 SMTP 外部发送测试。
- 不引入模板文件加载、模板热更新、队列、限流或批量发送。

## 主方案

1. 配置层新增 `[email]`：
   - `smtp_host`
   - `smtp_port`
   - `smtp_user`
   - `smtp_password`
   - `smtp_password_file`
   - `smtp_use_tls`
   - `from_name`
   - `timeout_seconds`
   - `max_retries`
   - `retry_delay_seconds`
2. `AppConfig::resolve_secrets` 支持 `email.smtp_password_file`。
3. `AppConfig::validate_sensitive_config` 在 production 中对配置了 SMTP host/user 的 email 配置要求 `smtp_password_file`。
4. `EmailSender` 在创建时做配置校验和 SMTP transport 构造；发送时做收件人校验、消息构建和重试。
5. `EmailVerificationScene` 与模板渲染函数提供固定验证码场景。
6. release 链路新增 `app_email_smtp_password` secret。

## 关键取舍

- 选择 `lettre` 而不是手写 SMTP：减少协议细节和 TLS 风险。
- 选择 rustls TLS feature：降低 Docker 静态构建中 native TLS 差异。
- 选择内联模板函数而不是模板引擎：当前只有固定验证码场景，引入 Jinja 类模板引擎会增加依赖和运行时复杂度。
- 配置缺省向后兼容：已有本地 runtime config 没有 `[email]` 时不阻断应用启动；只有创建 `EmailSender` 或 production 配置显式启用 SMTP 值时才 fail-fast。

## 禁止触碰边界

- 不修改 auth handler、verification service、frontend、数据库迁移。
- 不修改用户 ignored 的 `config/development.toml`。
- 不记录 SMTP 授权码原文。
