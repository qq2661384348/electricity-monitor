# ADR-0001: Email SMTP 使用 lettre async rustls

## 状态

Accepted

## 背景

邮件模块需要 SMTP SSL / STARTTLS、认证、MIME message 构建和 Tokio async 发送能力。手写 SMTP 协议会扩大安全与兼容风险。

## 决策

新增 `lettre` 作为 SMTP 客户端依赖，使用 Tokio async 与 rustls TLS feature。

## 影响

- 减少 SMTP/TLS 自研风险。
- 避免新增 native TLS 平台依赖，降低 Docker 静态构建差异。
- 后续如果需要真实 SMTP 集成测试，可直接复用 `EmailSender`。
