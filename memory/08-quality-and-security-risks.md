# Electricity Monitor 仓库记忆：质量与安全风险

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

## 当前质量风险

- 前端生产构建仍有大 chunk warning，主要集中在 `lib-react-dom`、`lib-react-router` 和 `vendor` 分块；这是性能治理项，不是功能阻塞项。
- `DashboardPage.tsx` 仍是高耦合页面容器，虽然 modal 已做懒加载，但 dashboard 数据装配和交互编排仍偏集中。
- `frontend/src/services/api.ts` 仍保留兼容 facade，后续需要继续缩小存在感，避免旧入口反向扩张。

## 当前供应链风险

- 前端依赖审计仍有未清零告警，当前主要集中在开发工具链与构建链的上游依赖：`eslint` 相关链路带来的 `minimatch`、`ajv`、`brace-expansion`、`flatted`，以及 `vite` / `vitest` / `typescript-eslint` 链路上的 `picomatch`、`rollup`。
- 这些前端告警当前通过 `dependency-audit` job 持续报告，但不阻断 PR；每次升级 Vite / ESLint / TypeScript ESLint 生态时都要复核是否已被上游关闭。
- `bun audit` 与 `cargo audit` 的结论都需要结合本仓库真实执行路径判断，但报告项不能因为“暂未利用”就停止跟踪。

## 仓库外部安全缺口

- 反向代理、TLS 终止、响应安全头和 WAF 仍依赖部署环境负责，不在仓库内实现。
- secret file 的实际权限、服务器侧凭据轮换、运维审计记录同样属于仓库外部控制面；没有外部证据时，不能在仓库里写成“已完成”。

## 与历史配置相关的风险

- 仓库曾出现过把真实敏感配置写入 git 历史的情况，因此必须继续避免把真实 secret 写回仓库。
- 真实 `QQ/JWT/DB` 凭据轮换仍依赖外部 secret file 或部署目标访问权；在没有外部证据前，不应在仓库中写成“已完成轮换”。

## 优先级方向

- 第一优先级：继续跟进前端工具链上游 advisory，尤其是 `vite` / `rollup` 与 `eslint` 依赖链。
- 第二优先级：进一步治理前端大 chunk，优先继续拆 dashboard 与公共 vendor。
- 第三优先级：在部署环境补齐反向代理、TLS、响应安全头与 secret file 权限控制。
