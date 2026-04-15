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

## 当前质量风险

- `DashboardPage.tsx` 仍是高耦合页面容器，虽然 modal 已做懒加载，但 dashboard 数据装配和交互编排仍偏集中。
- `frontend/src/services/api.ts` 仍保留兼容 facade，后续需要继续缩小存在感，避免旧入口反向扩张。
- 部署脚本的回滚路径仍需要在真实 Linux Docker 主机上做端到端演练，本地 readiness test 不能替代这类验证。

## 当前供应链风险

- 前端工具链上游 advisory 变化快；每次升级 `vite`、`eslint`、`typescript-eslint` 或其相关插件后，都要重新执行 `bun audit`，并确认 `overrides` 仍指向修复版本而不是过期锁定。
- Rust 依赖升级后仍要重新执行 `cargo audit -q`，不能因为当前已清零就把审计当成一次性动作。

## 仓库外部安全缺口

- 反向代理、TLS 终止、`Strict-Transport-Security` 与 WAF 仍依赖部署环境负责，不在仓库内实现。
- 服务器侧凭据轮换、运维审计记录同样属于仓库外部控制面；没有外部证据时，不能在仓库里写成“已完成”。

## 与历史配置相关的风险

- 仓库曾出现过把真实敏感配置写入 git 历史的情况，因此必须继续避免把真实 secret 写回仓库。
- 真实 `QQ/JWT/DB` 凭据轮换仍依赖外部 secret file 或部署目标访问权；在没有外部证据前，不应在仓库中写成“已完成轮换”。

## 优先级方向

- 第一优先级：继续降低 `DashboardPage.tsx` 与兼容 facade 的耦合，避免前端复杂度重新堆回入口层。
- 第二优先级：升级前端工具链时同步复核 `overrides`、`bun audit` 和 bundle budget，避免告警基线回退。
- 第三优先级：在部署环境补齐反向代理、TLS、`Strict-Transport-Security` 与 WAF。
