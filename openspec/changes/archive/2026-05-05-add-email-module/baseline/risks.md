# Risks

| ID | 风险 | 影响 | 缓解 |
| --- | --- | --- | --- |
| RISK-001 | 真实 SMTP 授权码进入 tracked 文件 | 凭据泄漏、需要轮换 | 模板只写非敏感值；密码走 secret file；spec 和文档不记录原文 |
| RISK-002 | 新邮件依赖破坏 Docker 静态构建 | release 构建失败 | 使用 lettre rustls async feature，避免 native TLS 新平台依赖 |
| RISK-003 | 新模块被误解为已接入业务流程 | 产品行为预期漂移 | 明确非目标：不接入 auth/API/通知调度；导出可复用 sender 供后续 change 使用 |
| RISK-004 | 生产配置模板新增 secret file 但 release 未挂载 | 生产启动或邮件发送失败 | 同步 `config/production.toml.example`、`deploy/release.env.example`、`compose.release.yml`、`deploy.sh` 和 secrets inventory |
| RISK-005 | SMTP 真实外部发送难以在 CI 自动验证 | 验证不足 | 单元测试覆盖构建/校验/重试逻辑；真实发送列为外部集成验证，需手动提供 secret |
