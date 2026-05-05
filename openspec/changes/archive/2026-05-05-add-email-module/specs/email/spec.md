# Delta for Email

## ADDED Requirements

### Requirement: Email Runtime Configuration

系统必须提供 `[email]` 运行时配置域，支持 SMTP host、port、user、password、password file、TLS 模式、from name、timeout、max retries 和 retry delay。

#### Scenario: Load email defaults from runtime config

- **GIVEN** 运行时 TOML 包含 `[email]`
- **WHEN** `AppConfig` 加载配置
- **THEN** 系统必须把 email 配置反序列化到 `AppConfig.email`
- **AND** 支持 `APP__EMAIL__<KEY>` 环境变量覆盖

#### Scenario: Resolve production SMTP password from secret file

- **GIVEN** production 配置提供 `email.smtp_password_file`
- **WHEN** `AppConfig` 解析 secret
- **THEN** 系统必须从该文件读取 SMTP password
- **AND** 不在日志或文档输出 password 原文

### Requirement: Async SMTP Email Sender

系统必须提供可复用 async SMTP sender，支持普通文本邮件、HTML 邮件、邮箱地址校验、发件人显示名和失败重试。

#### Scenario: Build and send a text email

- **GIVEN** 有效 email 配置、有效收件人、主题和文本正文
- **WHEN** 调用 `send_email`
- **THEN** 系统必须构造合法 MIME message 并通过 SMTP transport 发送

#### Scenario: Reject invalid recipient

- **GIVEN** 收件人地址格式无效
- **WHEN** 调用邮件发送
- **THEN** 系统必须返回 validation error
- **AND** 不触达 SMTP transport

### Requirement: Verification Email Templates

系统必须提供验证码邮件模板渲染能力，支持 `register`、`login`、`reset`、`reset_password`、`bind`、`unbind` 场景。

#### Scenario: Render login verification email

- **GIVEN** 验证码 `123456` 和场景 `login`
- **WHEN** 调用验证码邮件渲染
- **THEN** 系统必须返回包含验证码的主题、HTML 正文和纯文本正文

#### Scenario: Reject unsupported verification scene

- **GIVEN** 未支持的验证码场景
- **WHEN** 调用验证码邮件发送
- **THEN** 系统必须返回 validation error

### Requirement: Source-of-truth Synchronization

系统必须同步配置模板、release secret 链路、secrets inventory、部署文档和配置 memory，使新增 email 配置与生产 secret 注入规则一致。

#### Scenario: Production release has email secret

- **GIVEN** release 包使用 `deploy/release.env.example` 和 `deploy/compose.release.yml`
- **WHEN** 部署执行人准备 `.env` 与 secrets
- **THEN** 模板必须声明 `APP_EMAIL_SMTP_PASSWORD_SECRET_FILE`
- **AND** compose 必须把 secret 挂载到 `/run/secrets/app_email_smtp_password`

## MODIFIED Requirements

### Requirement: Production Sensitive Config Validation

生产环境敏感配置校验必须覆盖已有 DB/JWT/QQ secret file，并在 email SMTP 配置存在时覆盖 `email.smtp_password_file`。

#### Scenario: Production email config without secret file

- **GIVEN** production 配置包含 email SMTP host 或 user，但没有 `email.smtp_password_file`
- **WHEN** 执行 production 配置敏感校验
- **THEN** 系统必须返回配置错误

## REMOVED Requirements

- None.
