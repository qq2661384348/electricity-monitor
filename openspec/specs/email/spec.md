# Email Spec

## Runtime Configuration

系统提供 `[email]` 运行时配置域，支持：

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

`email.smtp_password` 是 SMTP 授权码，不能写入 tracked 模板或公开文档。生产环境通过 `email.smtp_password_file` / `APP__EMAIL__SMTP_PASSWORD_FILE` 读取 `/run/secrets/app_email_smtp_password`。

## SMTP Sender

系统提供可复用 async SMTP sender，支持：

- 普通文本邮件
- HTML 邮件
- multipart alternative 验证码邮件
- 发件人显示名
- 收件人邮箱格式校验
- 发送失败重试

`smtp_use_tls=true` 使用 implicit TLS；`smtp_use_tls=false` 使用 STARTTLS。SMTP transport 在发送阶段创建，避免应用启动或同步测试阶段要求 Tokio reactor。

## Verification Email

系统支持验证码邮件场景：

- `register`
- `login`
- `reset`
- `reset_password`
- `bind`
- `unbind`

验证码邮件必须包含主题、纯文本正文和 HTML 正文。模板渲染必须保留验证码原文，不输出 SMTP password。

## Boundaries

- 当前 email 模块是后端基础设施能力，不替换 QQ 验证码发送链路。
- 当前 email 模块不新增 HTTP API、前端页面、数据库 schema 或后台通知调度。
- 真实 SMTP 外部发送属于可选手动验证，不作为自动测试门禁。
