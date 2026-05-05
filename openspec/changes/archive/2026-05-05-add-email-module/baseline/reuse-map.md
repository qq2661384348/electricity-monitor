# Reuse Map

| 复用对象 | 当前职责 | 本 change 用法 |
| --- | --- | --- |
| `src/config/app.rs` | 聚合配置、解析 secret file、生产敏感配置校验 | 新增 `EmailConfig` 聚合、`smtp_password_file` 解析、生产 secret file 校验 |
| `src/config/*.rs` | 单配置域结构与默认值 | 新增 `src/config/email.rs`，沿用 serde + Default + 单元测试风格 |
| `src/infrastructure/notification/*` | 外部发送模块组织、错误类型、客户端构造与导出风格 | 新增 `src/infrastructure/email/*`，复用 `error.rs` / `sender.rs` / `templates.rs` / `mod.rs` 分层方式 |
| `thiserror` | 错误枚举 | 邮件错误分类 |
| `regex` | 已有正则依赖 | 邮箱地址轻量校验 |
| `tokio` | async runtime | SMTP 发送重试等待 |
| `deploy/*` secret 模式 | 生产 secret file 声明、挂载、权限校验 | 增加 email SMTP password secret |
| `memory/long-term/semantic/config-and-environments.md` | 配置真源记忆 | 同步 email 配置与 secret 链路 |
| `docs/guides/SECRETS_INVENTORY.md` | 生产 secrets 清单 | 增加 SMTP 授权码 |
