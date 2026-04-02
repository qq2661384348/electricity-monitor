# Electricity Monitor 仓库记忆：质量问题与安全问题基线

## 目的
- 这份文档专门记录当前仓库里已经确认存在、且值得后续任务直接复用的质量问题与安全问题。
- 只保留长期有效的项目知识，不记录机器私有信息，不记录本地数据库密码，不记录历史镜像源之类的临时环境细节。

## 当前真实质量问题
- `cargo clippy --all-targets -- -D warnings` 当前不能通过，稳定阻塞点在 `src/domain/services/notification_gate.rs`
- `scripts/check-architecture.ps1` 会把 `frontend/src/**/AGENTS.md` 的说明文本误报成 `@/services/api` 违规导入
- `frontend/package.json` 当前把 `@tailwindcss/vite` 放在 `dependencies`，导致构建链告警进入生产依赖审计面
- 前端生产构建已经持续出现多个 `>64 kB` 的 chunk warning，说明页面分块与公共库体积值得继续治理

## 当前真实安全问题
- `/api/auth/refresh` 未区分 access token 与 refresh token
- 前端 `auth-storage` 持久化 access token，会放大未来同源 XSS 的后果
- 后端全局 CORS 仍是 `Any / Any / Any`
- `admin.default_qq_number` 在生产环境下尚未强制要求偏离默认值

## 当前依赖与供应链问题
- 前端依赖审计当前命中 `7` 条 advisory：
  - 运行时直接依赖重点是 `axios` 与 `react-router-dom`
  - 构建链重点是 `rollup` 与 `picomatch`
- Rust 依赖审计当前命中 `5` 条漏洞与 `2` 条警告：
  - 重点是 `bytes`、`rkyv`、`rsa`、`rustls-webpki`、`time`
  - 另有 `bincode` unmaintained 与 `lru` unsound
- 这些发现说明项目已经具备自动审计能力，但依赖基线本身仍需升级与替换计划

## 已确认的误报与边界说明
- `scripts/check-architecture.ps1` 的当前失败不能直接视为前端源码回归，需要先用源码级 `rg` 复核 `*.ts` / `*.tsx`
- `react-router` 的部分 advisory 与 SSR / Server Action 相关；当前仓库前端是 SPA，适用性需要逐项判断，不能直接机械照抄上游标题
- `cargo audit` / `pnpm audit` 的上游告警需要结合当前代码路径继续做二次适用性判断，但这不会抵消“版本已脱离安全基线”的事实

## 已补齐的环境与工具链事实
- 项目当前已具备可执行的依赖审计路径：
  - `pnpm --dir frontend audit --prod`
  - `cargo audit`
- 若本地 PostgreSQL 密码与仓库默认开发配置不一致，应优先通过单次命令前的 `APP__DATABASE__PASSWORD` 环境变量覆盖来恢复后端契约测试

## 后续修复优先级建议
- 第一优先级：
  - 为 JWT 增加 token kind，并限制 `/api/auth/refresh` 只接受 refresh token
  - 停止把 access token 持久化到浏览器本地存储
  - 将 CORS 改为配置化 allowlist
  - 让生产环境在默认管理员 QQ 号未覆盖时 fail-fast
- 第二优先级：
  - 处理 `axios`、`react-router-dom`、`bytes`、`rkyv`、`rustls-webpki`、`time`、`lru` 的升级
  - 为 `bincode` 制定替换或风险接受策略
  - 把 `@tailwindcss/vite` 移到 `devDependencies`
- 第三优先级：
  - 修复 `notification_gate.rs` 的 clippy 告警
  - 修正架构守护脚本的误报逻辑
  - 继续优化前端构建分块
