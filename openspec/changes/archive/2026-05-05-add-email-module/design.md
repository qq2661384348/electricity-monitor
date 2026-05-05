# Design

## D-001 模块布局

```text
src/config/email.rs
src/infrastructure/email/
├── error.rs
├── mod.rs
├── sender.rs
└── templates.rs
```

`src/infrastructure/mod.rs` 只导出 `EmailSender` 和必要类型，保持与 `QQClient` 相同的基础设施层边界。

## D-002 配置结构

`EmailConfig` 字段：

- `smtp_host: String`
- `smtp_port: u16`
- `smtp_user: String`
- `smtp_password: String`
- `smtp_password_file: Option<String>`
- `smtp_use_tls: bool`
- `from_name: String`
- `timeout_seconds: u64`
- `max_retries: u32`
- `retry_delay_seconds: u64`

默认值只用于反序列化兼容；`EmailSender::new` 会验证必填字段。生产敏感配置校验只在 email SMTP host/user 被配置时要求 secret file，避免旧部署在未使用邮件模块时被无意义阻断。

## D-003 SMTP transport

- `smtp_use_tls=true`：使用 implicit TLS 连接，适配 QQ 邮箱 465。
- `smtp_use_tls=false`：使用 STARTTLS 连接，仍避免明文认证。
- `timeout_seconds` 传给 SMTP transport。
- credentials 使用 `smtp_user` + 解析后的 `smtp_password`。

## D-004 错误模型

`EmailError` 至少区分：

- `Config`
- `Validation`
- `Template`
- `Address`
- `Build`
- `Transport`

错误信息不携带 password。

## D-005 模板策略

固定验证码场景由 Rust 函数生成：

- 主题：`【{from_name}】{场景标题}`
- HTML：包含验证码、使用说明和安全提醒
- Text：同等语义的纯文本

这样避免新增模板引擎，同时保留后续替换为文件模板的扩展点：`templates.rs` 是唯一模板出口。

## D-006 Secret 与部署

生产模板与 release 采用：

- `email.smtp_password_file = "/run/secrets/app_email_smtp_password"`
- `APP_EMAIL_SMTP_PASSWORD_SECRET_FILE=./secrets/app_email_smtp_password`
- compose secret source/target 均为 `app_email_smtp_password`

部署脚本把该 secret 纳入 owner-only 权限校验。

## D-007 测试策略

- 配置测试：默认值、secret file 覆盖、production 缺失 email secret file 失败。
- 邮件模块测试：配置校验、邮箱校验、验证码场景解析、模板包含验证码、消息构建不触发真实 SMTP。
- 不做真实 SMTP 外部发送自动测试。
