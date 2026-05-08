# Baseline

## B-001 当前仓库后端结构

- 后端是 Rust + Axum 项目，配置入口是 `src/config/app.rs`，各配置域在 `src/config/*.rs` 中拆分并由 `AppConfig` 聚合。
- 外部发送类能力当前集中在 `src/infrastructure/notification/`，其中 `QQClient` 负责 NapCat HTTP 私聊发送；仓库还没有 SMTP / Email 模块。
- `src/infrastructure/external/` 只封装 HTTP 客户端，不适合复用到 SMTP。

## B-002 当前配置与 secret 规则

- 运行时配置加载顺序固定为环境命名 TOML 后叠加 `APP__<SECTION>__<KEY>` 环境变量。
- 生产敏感配置必须通过 `*_FILE` 链路注入，不在 `*.toml.example`、`.env.example` 或文档中保存真实 secret。
- `config/development.toml` 是本地忽略文件，不能把用户本地配置作为仓库真源改写。

## B-003 当前部署链路

- 生产发布主线是 `.github/workflows/docker-build.yml` 与 `deploy/`。
- release 包通过 `deploy/release.env.example` 声明宿主机 secret 文件路径，通过 `deploy/compose.release.yml` 挂载到 `/run/secrets/*`。
- `deploy/deploy.sh` 会校验 secret file 是否存在且权限收紧到 owner-only。

## B-004 参考项目邮件模块

参考模块来自用户提供的既有 Python 项目邮件实现；OpenSpec 只记录可复用能力，不记录本地绝对路径，避免面向开源协作时泄露开发环境细节。

- `sender.py`：支持 SMTP SSL、SMTP + STARTTLS、发件人显示名、收件人邮箱校验、异步封装、重试、普通邮件、模板邮件、验证码邮件和批量发送。
- `templates.py`：用模板缓存渲染 `register/login/reset/bind/unbind` 五类验证码邮件，模板内容以 `Subject:` 行拆分主题和正文。
- `exceptions.py`：区分配置、发送、模板、校验、限流、超时、认证等错误。
- 参考配置包含 SMTP host/port/user/password/use_tls/from_name/timeout/max_retries/retry_delay。

## B-005 外部依赖调研

- Rust 邮件发送需要新增 SMTP 客户端依赖；`lettre` 是当前主流 Rust 邮件库，`docs.rs` / `crates.io` 当前 latest 为 `0.11.x`，支持 Tokio async SMTP 与 rustls TLS。
- 为减少系统依赖与 Docker 静态构建风险，优先使用 `tokio1-rustls-tls`，避免引入 native TLS 额外平台差异。

## B-006 足够好的结果草案

- 新增可复用邮件基础设施模块，不接入现有 QQ 验证码登录链路、不新增 HTTP API、不改变数据库 schema。
- 新模块能通过配置创建 SMTP sender，支持普通文本/HTML 邮件与验证码邮件场景。
- 配置模板、生产 secret 链路和配置真源记忆同步更新。
- 单元测试覆盖配置解析、secret file 覆盖、基础消息构建、验证码场景校验和重试配置边界。
