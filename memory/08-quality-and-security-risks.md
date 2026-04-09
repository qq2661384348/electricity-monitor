# Electricity Monitor 仓库记忆：质量与安全风险

## 目的

- 本文件只记录已经确认存在、且值得后续任务直接复用的长期质量风险与安全风险。
- 不记录机器私有信息、本地密码、一次性调试信息或单次审计统计结果。

## 当前质量风险

- `cargo clippy --all-targets -- -D warnings` 仍不能稳定通过，主要阻塞点在 `src/domain/services/notification_gate.rs`。
- `scripts/check-architecture.ps1` 对 `frontend/src/**/AGENTS.md` 的文本说明存在误报可能；当它报出 `@/services/api` 回归时，需要再用源码级 `rg` 复核 `*.ts` / `*.tsx`。
- 前端生产构建仍持续出现大 chunk warning，说明页面分块与公共库体积仍需要治理。

## 当前安全风险

- `/api/auth/refresh` 尚未区分 access token 与 refresh token。
- 前端 `auth-storage` 仍持久化 access token，会放大未来同源 XSS 的后果。
- 后端全局 CORS 仍是宽放行配置。
- `admin.default_qq_number` 在生产环境下尚未强制要求偏离默认值。

## 当前供应链风险

- 前端与 Rust 依赖都需要持续通过 `bun audit` 与 `cargo audit` 复核。
- 当前需要重点关注的前端依赖簇包括 `axios`、`react-router-dom`、`rollup` 与 `picomatch`。
- 当前需要重点关注的 Rust 依赖簇包括 `bytes`、`rkyv`、`rsa`、`rustls-webpki`、`time`、`bincode` 与 `lru`。
- 上游 advisory 的适用性需要结合本仓库真实代码路径判断，但这不意味着可以跳过升级与风险接受决策。

## 与历史配置相关的风险

- 仓库曾出现过把真实敏感配置写入 git 历史的情况，因此必须继续避免把真实 secret 写回仓库。
- 真实 `QQ/JWT/DB` 凭据轮换仍依赖外部 secret file 或部署目标访问权；在没有外部证据前，不应在仓库中写成“已完成轮换”。

## 优先级方向

- 第一优先级：修复 token kind、浏览器持久化 access token、宽 CORS 和生产默认管理员配置问题。
- 第二优先级：处理前后端依赖升级、替换或风险接受策略。
- 第三优先级：修复 clippy 阻塞点、校正架构守护误报逻辑，并继续优化前端构建分块。
