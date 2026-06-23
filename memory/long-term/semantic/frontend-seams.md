---
type: semantic
status: verified
scope: 前端可维护性接缝
updated_at: 2026-06-23
verified_at: 2026-06-23
sources:
  - frontend/src/App.tsx
  - frontend/src/shared/api/http-client.ts
  - frontend/src/entities/public-config/api/publicConfigApi.ts
  - frontend/src/features/public-config/model/usePublicConfig.ts
  - frontend/src/features
  - frontend/src/entities
summary: 前端启动、会话、HTTP、page/feature/entity 和可复用 UI 接缝
---

# Electricity Monitor 前端可维护性接缝

## 启动与会话接缝

- 前端启动骨架固定为 `frontend/src/main.tsx` -> `frontend/src/App.tsx` -> `frontend/src/routes.tsx`。
- 浏览器端只在内存持有 access token。
- refresh token 只存在于后端签发的 HTTPOnly Cookie，不通过 JSON 暴露给前端代码。
- 页面刷新后的会话恢复统一走 `/api/auth/refresh`。
- 并发 401 回收敛到单个 in-flight refresh promise，避免同一轮失效触发刷新风暴。

## HTTP 与兼容接缝

- `frontend/src/shared/api/http-client.ts` 是真实 HTTP client 真源，统一处理 `/api` 前缀、`withCredentials`、Bearer token 注入、refresh 和 401 重放。
- `frontend/src/services/api.ts` 只是兼容 facade，不应重新演化成新的 API 真源。
- 新增接口优先落在 `shared/api`、`entities/*/api` 或 `features/*/api`，不要直接在页面或组件里散落 Axios 调用。
- 后端 HTTP JSON 中的 `roomid` 对前端是字符串，不是 number；React Query key、绑定请求体和路径树叶子节点都应保留字符串，避免 18 位 Upay RoomID 在浏览器中丢精度。

## 公开运行时配置接缝

- `/api/public-config` 的前端访问收敛在 `frontend/src/entities/public-config/api/publicConfigApi.ts`，React Query hook 收敛在 `frontend/src/features/public-config/model/usePublicConfig.ts`。
- 登录页、登录弹窗、验证码弹窗、首页教程和公告中的登录模式可用性、机器人 QQ、管理员 QQ、第三方验证码参数与登录验证码长度都应读取公开配置；不要在 UI 中硬编码机器人号、管理员号或 6 位验证码长度。
- `frontend/src/entities/public-config/api/publicConfigApi.ts` 中的 fallback 只用于接口短暂不可用时维持旧默认体验；真实运行时仍依赖后端配置校验保证机器人 QQ 与管理员 QQ 非空。

## 认证登录接缝

- QQ / 邮箱登录的标识规范化、校验和展示文案收敛在 `frontend/src/features/auth-login/model/loginIdentity.ts`。
- 登录 API 调用统一走 `frontend/src/features/auth-login/api/authApi.ts`，请求体同时发送 `login_mode`、`identifier` 与当前渠道兼容字段，保持新旧后端调用方兼容。

## 页面 / feature / entity 接缝

- 页面层优先承担容器和布局装配职责，复杂流程下沉到 `features/`。
- `features/` 负责流程级 `api / model / ui / index.ts` 组织、query invalidation、mutation 编排和页面交互状态。
- `entities/` 负责单领域 API、轻量领域转换和稳定公共出口，不承接页面状态、路由跳转或跨实体编排。

## 可复用 UI 接缝

- `frontend/src/components/ui/comic-modal/` 已采用 compound components 模式，是当前较成熟的可复用 UI 资产。
- 前端后续升级应优先复用现有基础设施，而不是重做一套新的 UI 组件模式。
