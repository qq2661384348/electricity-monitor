# Constraints

## C-001 最小范围

- 本 change 只新增后端邮件基础设施能力，不改变现有认证、通知、前端、数据库和 API 契约。
- 不把 QQ 通知替换为邮件通知，不新增 `EmailNotificationSender` 业务编排。

## C-002 配置安全

- 用户提供的 SMTP 授权码属于真实 secret，不能写入 tracked 文件、spec 工件、文档或日志。
- 生产配置必须使用 `email.smtp_password_file` / `APP__EMAIL__SMTP_PASSWORD_FILE`，release 包必须挂载对应 secret。
- 开发模板可以保留非敏感默认值：SMTP host、port、user、TLS 开关、from name、timeout、retry 参数；密码只能留空或占位。

## C-003 技术栈

- 后端保持 Rust + Tokio async 模型。
- 新增依赖必须服务于 SMTP 发送闭环；不引入模板引擎，除非验证码模板无法用项目内小型渲染函数表达。
- 代码注释和文档默认中文，公开可见文本不包含模型来源归因。

## C-004 验证

- 至少运行 `cargo fmt --check` 与目标 `cargo test --lib`。
- 因新增依赖和后端代码，若环境允许，应运行 `cargo clippy --all-targets -- -D warnings`。
- 因修改部署 secret 链路，应做 `bash -n deploy/deploy.sh` 与 compose 文件引用扫描；Docker 可用时再做 compose config。
