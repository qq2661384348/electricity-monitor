# Unknowns

## 会改变实现路径的关键未知

- None.

## 非关键未知

| ID | 未知项 | 当前处理 |
| --- | --- | --- |
| U-001 | 后续是否用邮件替代或补充 QQ 验证码链路 | 本 change 不接入业务流程；后续若需要，另开 change 定义 API、前端、验证码存储与风控 |
| U-002 | 是否需要真实 SMTP 集成测试 | 本 change 提供模块和配置；真实发送依赖外部账号与 secret，不作为自动门禁 |

## 已拍定 / 由现有约束决定的决策

| ID | 决策 | 来源 |
| --- | --- | --- |
| DEC-001 | 不把 SMTP 授权码原文写入仓库 | 项目 secrets 规则与安全红线 |
| DEC-002 | 最小实现为后端可复用基础设施模块，不改业务流程 | 用户目标“添加邮件模块”与 Surgical Changes |
| DEC-003 | 使用 `lettre` async SMTP + rustls TLS | Rust async SMTP 需求与 Docker 构建约束 |
