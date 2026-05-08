# Requirements

## 1. 需求汇总

本 change 要为 Electricity Monitor 后端新增一个可复用邮件模块，参考用户提供的既有 Python 邮件模块的 SMTP 发送、重试、邮箱校验和验证码模板能力，同时把所需配置纳入当前项目配置模板、生产 secret 链路和配置真源记忆。

## 2. 已确认信息

- 参考项目邮件模块提供 SMTP 发送、重试、邮箱校验和验证码模板能力；OpenSpec 不记录本地绝对路径。
- 用户提供的非敏感配置默认值包括 `smtp.qq.com`、`465`、`cogniaegis@qq.com`、`smtp_use_tls=true`、`from_name=CogniAegis`、`timeout=30`、`max_retries=3`、`retry_delay=2`。
- 用户提供的 SMTP password 是真实 secret，按当前项目规则不能写入 tracked 模板或工件。
- 当前项目配置真源是 `src/config/app.rs`、`config/*.toml.example`、生产 release secret 链路和 `memory/long-term/semantic/config-and-environments.md`。

## 3. 待确认信息

### 3.1 会改变实现路径的关键未知

- None.

### 3.2 非关键未知

- 是否真实发送一封邮件验证 SMTP 账号：需要外部网络和 secret，不纳入自动验收。

### 3.3 明确选项与已拍定决策

- 选项 A：只新增邮件基础设施模块，不接入业务流程。本 change 采用。
- 选项 B：同时改造登录/注册/验证码流程接入邮件。需要 API、前端、验证码存储、风控和兼容策略，超出本 change。

## 4. 主要功能拆分

### 功能 1：Email 配置域

- 功能描述：新增 `[email]` 配置域，支持 SMTP host、port、user、password/password_file、TLS、from name、timeout、max retries、retry delay。
- 详细分析：配置结构应由 `AppConfig` 聚合；secret file 解析复用现有 `read_secret_file`；生产环境如果配置了 email SMTP 发送，则密码必须来自 secret file。
- 边界与约束：不在模板写真实 password；不启用全局 `try_parsing(true)`；保留 `APP__EMAIL__...` 覆盖链。
- 复用与依赖：复用 `src/config` 模式、生产 secret 校验模式和 config crate。
- 风险与待确认：本地 ignored `config/development.toml` 可能缺少 `[email]`，因此 `AppConfig` 应对缺省 email 配置保持向后兼容，实际创建 sender 时再 fail-fast。

### 功能 2：SMTP 邮件发送器

- 功能描述：新增 async SMTP sender，支持普通文本/HTML 邮件、发件人显示名、收件人邮箱校验、发送失败重试。
- 详细分析：Rust 侧使用 `lettre` 的 Tokio async SMTP transport；`smtp_use_tls=true` 对应 465 implicit TLS，false 对应 STARTTLS，避免明文认证。
- 边界与约束：不做连接池调优、批量发送、性能统计或 rate limit；不输出 password。
- 复用与依赖：复用 `tokio`、`thiserror`、`regex`，新增 `lettre`。
- 风险与待确认：真实 SMTP 错误只能在外部集成环境验证。

### 功能 3：验证码邮件模板

- 功能描述：支持 `register/login/reset/reset_password/bind/unbind` 验证码场景，生成主题、纯文本正文和 HTML 正文。
- 详细分析：参考项目使用 Jinja 模板文件；本项目为减少依赖，使用 Rust 常量/函数渲染场景化模板。
- 边界与约束：不新增模板文件加载器和模板热重载；验证码只校验非空数字，避免与当前 `[verification].code_length` 配置冲突。
- 复用与依赖：复用 email sender 的 HTML/文本发送能力。
- 风险与待确认：后续若需要运营可编辑模板，应另开 change 引入模板文件和渲染引擎。

### 功能 4：配置真源和部署同步

- 功能描述：同步开发/生产配置模板、release `.env.example`、compose secret、部署校验、secrets inventory、部署说明和 memory 配置真源。
- 详细分析：生产 `config/production.toml.example` 和 release compose 需要指向 `/run/secrets/app_email_smtp_password`；部署脚本需要校验宿主机 secret file 权限。
- 边界与约束：不把真实 secret 写入任何 tracked 文件。
- 复用与依赖：复用现有 `APP_*_SECRET_FILE` 约定。
- 风险与待确认：新增 secret file 会提高生产部署前置要求，需在文档中显式列出。

## 5. 全局约束与边界

- 不改变现有 QQ 验证码发送契约。
- 不新增 HTTP endpoint、前端页面、数据库表或后台任务。
- 不把用户提供的 SMTP password 原文落盘。
- 不引入模板引擎、队列系统或限流系统。

## 6. 复用策略与避免重复建设说明

- 配置、secret file、部署和记忆同步复用当前项目真源，不新增第二套 `.env` 解析。
- SMTP 发送使用成熟 Rust 邮件库，不手写 SMTP 协议。
- 邮件模板先用最小函数渲染，避免为固定验证码场景引入模板运行时。

## 7. 非目标与最小范围

- 非目标：邮件登录、邮箱绑定业务、密码重置流程、邮件通知调度、批量营销邮件、真实 SMTP 集成测试。
- 最小范围：可创建 sender、可构建并发送普通邮件与验证码邮件、配置可解析、生产 secret 链路完整、单元测试覆盖关键逻辑。

## 8. 足够好的结果与验收思路

- `cargo test --lib` 覆盖新增配置和邮件模块。
- `cargo fmt --check` 通过。
- `cargo clippy --all-targets -- -D warnings` 在环境允许时通过。
- 部署 secret 文件引用一致，`deploy/deploy.sh` shell 语法通过。
- 文档和 memory 中配置真源与 secrets inventory 已同步。

## 9. 额外文档与真源同步要求

- 更新 `memory/long-term/semantic/config-and-environments.md`。
- 更新 `docs/guides/SECRETS_INVENTORY.md`。
- 因 release secret 链路改变，同步 `README.md`、`docs/guides/DOCKER_DEPLOYMENT.md`、`deploy/README.md`、`memory/long-term/procedural/deploy-and-release.md`。

## 10. 进入 proposal 的前置条件

- baseline 完成，关键 unknowns 无阻塞。
- 技术栈确认：Rust + lettre async SMTP + rustls。
- 安全边界确认：SMTP password 不进入 tracked 文件。
