---
type: semantic
status: verified
scope: 长期质量风险与安全风险
updated_at: 2026-05-08
verified_at: 2026-05-08
sources:
  - docs/guides/TECHNICAL_DEBT.md
  - docs/guides/SECRETS_INVENTORY.md
  - .github/workflows/ci.yml
  - frontend/package.json
  - src/handlers/auth.rs
  - src/handlers/binding.rs
  - src/domain/services/rate_limiter.rs
  - tests/contracts/auth_integration_test.rs
  - tests/contracts/send_verification_code_integration_test.rs
  - src/infrastructure/repositories/electricity_history_repository.rs
  - tests/runtime/release_readiness_test.rs
summary: 已关闭风险、当前质量风险、供应链风险、仓库历史清理状态和仓库外部安全边界
---

# Electricity Monitor 质量与安全风险

## 目的

- 本文件只记录已经确认存在、且值得后续任务直接复用的长期质量风险与安全风险。
- 不记录机器私有信息、本地密码、一次性调试信息或单次审计统计结果。

## 已关闭的长期风险

- `cargo clippy --all-targets -- -D warnings` 已恢复为稳定通过，`NotificationGate` 不再是默认门禁阻塞点。
- `scripts/check-architecture.ps1` 已改为只扫描 `*.ts` / `*.tsx`，不会再因为 `AGENTS.md` 文本误报前端导入回退。
- `/api/auth/refresh` 已显式区分 access token 与 refresh token，受保护接口不会接受 refresh token 冒充 Bearer。
- 前端 access token 已从持久化存储移除，只在内存里保存；refresh token 只通过 HTTPOnly Cookie 往返。
- 后端 CORS 已改成配置驱动白名单，并为 cookie 会话开启 `allow_credentials(true)`。
- `admin.default_qq_number` 在生产环境下已经要求显式真实值，模板占位值不会再触发管理员权限。
- Rust 依赖审计当前可通过 `cargo audit -q`。
- 前端依赖审计当前可通过 `bun audit`；前端工具链升级后的传递依赖修复由 `frontend/package.json` 中的 `overrides` 负责固化。
- 前端生产构建不再保留默认 chunk warning 残留；真实 JS chunk 上限由 `frontend/scripts/check-bundle-budgets.ts` 持续校验。
- 后端统一响应安全头已收敛到应用层与 `deploy/smoke.targets` 共享契约；release smoke 与 readiness test 会同时校验这些头。
- release 部署脚本已对 secret file 做 owner-only 权限校验，过宽权限会直接阻断部署。
- `/api/auth/send-verification-code` 已要求先消费一次性 captcha token，不能再在缺失图形验证码校验时直接触达 QQ 机器人。
- `/api/auth/send-verification-code` 已增加应用侧 Redis 固定窗口配额，按全局、客户端来源和发送目标限制公开验证码发送入口。
- 普通用户创建 `/api/bindings` 绑定不再需要绑定码；风险边界转移为必须保持 `/api/bindings` 登录鉴权、房间存在性校验，以及 `/api/rooms/by-path`、`/api/rooms/by-hash` 未绑定拒绝读取详情。
- 电费抓取 HTTP 客户端已恢复 HTTPS 证书校验，生产路径不再接受无效证书。
- `/api/rooms/by-path` 与 `/api/rooms/by-hash` 已恢复为房间详情授权读取，未绑定普通用户不能再通过路径或哈希查询直接读取电费余额和阈值。
- 电费历史快照已改为数据库侧 `INSERT ... SELECT`；每小时任务不再先把所有活跃房间加载到 Rust 堆并构造逐条历史记录，release readiness test 会防止这类容器 RSS 高水位风险回退。
- 冷启动不再对全量 active room 执行 cache warm，也不再维护常驻 `flagged_rooms_cache`；路径树初始化改为最小字段投影，避免把完整 `Room` 和临时 `RoomData` 全量搬进启动堆。
- 本地和 `origin/master` 历史已重写并验证不再包含 `config/development.toml`、`config/production.toml` 或 `config/default.toml` 这类运行时配置文件路径。
- 2026-05-06 对本地可达 git 历史做了凭据形态扫描，未发现真实 `QQ/JWT/DB/SMTP` 凭据；命中内容为示例值、测试值、文档占位、公开 API URL 或依赖包示例 JWT。
- 远端仓库已删除重建，并已重新推送清理后的 `master`；远端旧历史残留不再作为当前仓库剩余风险记录。
- 旧绑定码机制已移除；若未来导入旧数据库备份，仍应按 `user_room_bindings` 是否符合真实账号关系审计绑定数据。

## 当前质量风险

- `DashboardPage.tsx` 仍是高耦合页面容器，虽然 modal 已做懒加载，但 dashboard 数据装配和交互编排仍偏集中。
- `frontend/src/services/api.ts` 仍保留兼容 facade，后续需要继续缩小存在感，避免旧入口反向扩张。
- 部署脚本的回滚路径仍需要在真实 Linux Docker 主机上做端到端演练，本地 readiness test 不能替代这类验证。

## 当前供应链风险

- 前端工具链上游 advisory 变化快；每次升级 `vite`、`eslint`、`typescript-eslint` 或其相关插件后，都要重新执行 `bun audit`，并确认 `overrides` 仍指向修复版本而不是过期锁定。
- Rust 依赖升级后仍要重新执行 `cargo audit -q`，不能因为当前已清零就把审计当成一次性动作。

## 仓库外部安全缺口

- 反向代理、TLS 终止、`Strict-Transport-Security` 与 WAF 仍依赖部署环境负责，不在仓库内实现。
- 验证码发送的客户端来源配额依赖生产反向代理正确清洗 `X-Forwarded-For`、`X-Real-IP` 或 `CF-Connecting-IP`，否则只能作为成本抬升，不能替代边缘侧限流。
- 服务器侧凭据轮换、运维审计记录同样属于仓库外部控制面；没有外部证据时，不能在仓库里写成“已完成”。

## 与历史配置相关的风险

- 仓库曾出现过把真实敏感配置写入 git 历史的情况；本地与重建后的远端 `master` 已完成清理并重新推送，但仍必须继续避免把真实 secret 写回仓库。
- 2026-05-06 凭据扫描未发现可达历史中仍保留真实 `QQ/JWT/DB/SMTP` 凭据，因此仓库证据不再要求强制轮换这些凭据；若外部环境曾复用过历史中出现过的值，仍可由运维侧主动轮换。

## 优先级方向

- 第一优先级：继续降低 `DashboardPage.tsx` 与兼容 facade 的耦合，避免前端复杂度重新堆回入口层。
- 第二优先级：升级前端工具链时同步复核 `overrides`、`bun audit` 和 bundle budget，避免告警基线回退。
- 第三优先级：在部署环境补齐反向代理、TLS、`Strict-Transport-Security`、边缘侧限流与 WAF。
