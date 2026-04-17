---
type: decision
status: active
scope: 浏览器鉴权会话模型
created_at: 2026-04-17
updated_at: 2026-04-17
sources:
  - src/handlers/auth.rs
  - frontend/src/shared/api/http-client.ts
  - frontend/src/stores/authStore.ts
  - docs/api/API_REFERENCE.md
summary: 浏览器端固定为内存 access token 加 HTTPOnly refresh cookie 的会话模型
superseded_by:
---

# 浏览器会话固定为内存 access token + HTTPOnly refresh cookie

## 背景

- 旧会话模型存在 access token 落地持久化、refresh token 经 JSON 往返的风险。
- 当前前后端已经统一到 Bearer access token 和 cookie refresh 的双层模型。

## 目标

- 降低浏览器端 token 暴露面。
- 收敛 refresh 契约，避免业务代码直接处理 refresh token。
- 让页面刷新后的会话恢复路径统一可控。

## 候选方案

### 方案 A

- access token 仅保存在内存中，refresh token 只通过 HTTPOnly Cookie 往返。

### 方案 B

- 在浏览器持久化 access token，或继续通过 JSON 暴露 refresh token。

## 最终选择

- 选择方案 A，采用内存 access token + HTTPOnly refresh cookie。

## 选择理由

- 该模型能缩小浏览器侧 token 暴露面，并把 refresh 边界收敛回后端和统一 HTTP client。
- `/api/auth/refresh`、`withCredentials`、单次 refresh promise 和 401 重放已经围绕该模型落地，继续保留旧协议只会增加复杂度。

## 后果与影响

- refresh token 不再暴露给前端业务代码。
- 页面刷新后的会话恢复必须走 `/api/auth/refresh`。
- 前端不再把 access token 写回 `localStorage`、`sessionStorage` 或其他持久化存储。

## 关联长期记忆

- `../long-term/semantic/auth-session-and-cors.md`
- `../long-term/semantic/frontend-architecture.md`

## 后续动作

- [ ] 后续若调整鉴权链路，继续验证 Bearer access token、HTTPOnly cookie、401 重放和 logout 清理逻辑保持一致。
