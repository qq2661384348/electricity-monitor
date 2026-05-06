# API 参考文档

## 概述

本文档说明 Electricity Monitor 后端当前对外提供的 REST API v1.1。

**基础 URL**: `http://localhost:8000/api`（开发环境）  
**认证方式**: access token Bearer + refresh token HTTPOnly Cookie  
**响应格式**: JSON

## 健康检查

### 基础健康检查

**端点**: `GET /api/health`  
**认证**: 无需认证  
**描述**: 检查服务是否运行

#### 请求示例

```bash
curl http://localhost:8000/api/health
```

#### 响应示例

```json
{
  "status": "ok",
  "message": "Service is healthy"
}
```

#### 响应字段

| 字段 | 类型 | 说明 |
|------|------|------|
| status | string | 服务状态，值为 "ok" |
| message | string | 状态描述 |

---

### 数据库健康检查

**端点**: `GET /api/health/db`  
**认证**: 无需认证  
**描述**: 检查服务和数据库连接状态

#### 请求示例

```bash
curl http://localhost:8000/api/health/db
```

#### 响应示例

**成功响应**（200 OK）:
```json
{
  "status": "ok",
  "database": "connected",
  "message": "Service and database are healthy"
}
```

**失败响应**（503 Service Unavailable）:
```json
{
  "error": "数据库操作失败",
  "message": "Database error: connection refused"
}
```

---

## 公开运行时配置

### 读取公开配置

- **端点**: `GET /api/public-config`
- **认证**: 无需认证
- **描述**: 返回前端需要展示或复用的非敏感运行时配置，包括可用登录模式、机器人 QQ、管理员 QQ、第三方图形验证码参数和登录验证码参数。该接口不会暴露 NapCat Bearer token、SMTP 授权码、JWT secret、数据库密码等敏感字段。

#### 请求示例

```bash
curl http://localhost:8000/api/public-config
```

#### 响应示例

```json
{
  "notification": {
    "qq_bot_public_qq_number": "100000002",
    "admin_qq_number": "100000001"
  },
  "captcha": {
    "api_url": "https://v2.xxapi.cn/api/captcha",
    "request_timeout_seconds": 5,
    "token_expire_seconds": 60,
    "captcha_type": "math",
    "width": 300,
    "height": 100,
    "options": 2
  },
  "verification": {
    "code_length": 6,
    "expire_seconds": 300
  },
  "auth": {
    "login_modes": ["qq", "email"],
    "email_login_enabled": true
  }
}
```

#### 响应字段

| 字段 | 类型 | 说明 |
|------|------|------|
| notification.qq_bot_public_qq_number | string | 部署者手动配置的机器人 QQ 号，用于前端引导用户添加好友 |
| notification.admin_qq_number | string | 管理员 QQ 号，用于前端公告和异常引导 |
| captcha.api_url | string | 第三方图形验证码生成与校验 API 地址 |
| captcha.request_timeout_seconds | number | 第三方验证码 HTTP 请求超时时间，单位秒 |
| captcha.token_expire_seconds | number | `/api/captcha/verify` 成功后签发的一次性 `captcha_token` 有效期，单位秒 |
| captcha.captcha_type | string | 图形验证码类型，支持 `string`、`math`、`digit` |
| captcha.width | number | 前端生成验证码图片时传给第三方的宽度 |
| captcha.height | number | 前端生成验证码图片时传给第三方的高度 |
| captcha.options | number | 前端生成验证码图片时传给第三方的难度等级，支持 `1`、`2`、`3` |
| verification.code_length | number | 登录验证码长度 |
| verification.expire_seconds | number | 登录验证码在 Redis 中的有效期，单位秒 |
| auth.login_modes | string[] | 当前可选登录模式；始终包含 `qq`，SMTP 邮件发送完整配置后包含 `email` |
| auth.email_login_enabled | boolean | 邮箱登录是否可用，由 SMTP host、user 和有效授权码是否完整决定 |

> `qq_bot_public_qq_number` 必须由部署者手动填写，不能从 NapCat 登录信息自动推断。

---

## 认证

当前认证链路采用“双 token”模型：

- access token：通过 JSON 响应返回，由前端只保存在内存中，并以 `Authorization: Bearer <token>` 调用受保护接口
- refresh token：只通过 HTTPOnly Cookie `refresh_token` 下发，不出现在 JSON 响应里

### 发送验证码

**端点**: `POST /api/auth/send-verification-code`  
**认证**: 无需认证  
**描述**: 消费 `/api/captcha/verify` 返回的一次性 `captcha_token` 后，按 `login_mode` 向 QQ 机器人或邮箱发送一次性登录验证码。缺省 `login_mode` 时按 `qq` 处理，用于兼容旧前端。

#### 请求体

QQ 登录：

```json
{
  "login_mode": "qq",
  "identifier": "123456789",
  "qq_number": "123456789",
  "captcha_token": "captcha-token-from-api-captcha-verify"
}
```

邮箱登录：

```json
{
  "login_mode": "email",
  "identifier": "student@example.com",
  "email": "student@example.com",
  "captcha_token": "captcha-token-from-api-captcha-verify"
}
```

> `identifier` 是统一登录标识。QQ 登录会兼容旧字段 `qq_number`，邮箱登录会兼容字段 `email`。`captcha_token` 为必填项，且只能使用一次。缺失、过期或重复使用都会被拒绝，服务端不会继续调用 QQ 机器人或 SMTP 邮件发送器。服务端还会在发送前后执行 Redis 固定窗口限流，覆盖全局发送量、客户端标识和目标登录标识；超限时返回 429，且不会继续触达 QQ 机器人或 SMTP。

#### 响应示例

QQ 登录：

```json
{
  "message": "验证码已发送",
  "login_mode": "qq",
  "identifier": "123456789",
  "qq_number": "123456789",
  "email": null
}
```

邮箱登录：

```json
{
  "message": "验证码已发送",
  "login_mode": "email",
  "identifier": "student@example.com",
  "qq_number": null,
  "email": "student@example.com"
}
```

---

### 校验图形验证码

**端点**: `POST /api/captcha/verify`
**认证**: 无需认证
**描述**: 校验第三方图形验证码答案，成功后返回一个只用于发送登录验证码的一次性 token。

#### 请求体

```json
{
  "id": "captcha-id",
  "key": "42",
  "type": "math"
}
```

> `type` 应与 `GET /api/public-config` 返回的 `captcha.captcha_type` 保持一致。

#### 响应示例

```json
{
  "success": true,
  "message": "验证通过",
  "code": "VERIFY_SUCCESS",
  "token": "one-time-captcha-token"
}
```

> `token` 只用于随后调用 `POST /api/auth/send-verification-code`，有效期短且消费后失效。

---

### 验证验证码并登录

**端点**: `POST /api/auth/verify-and-login`  
**认证**: 无需认证  
**描述**: 校验登录验证码，按 `login_mode` 创建或查找用户，并签发 access token 与 refresh cookie。QQ 登录仍可按 `admin.default_qq_number` 提升管理员；邮箱登录本轮始终为普通用户，不启用管理员提升。

#### 请求体

QQ 登录：

```json
{
  "login_mode": "qq",
  "identifier": "123456789",
  "qq_number": "123456789",
  "code": "123456"
}
```

邮箱登录：

```json
{
  "login_mode": "email",
  "identifier": "student@example.com",
  "email": "student@example.com",
  "code": "123456"
}
```

> `code` 长度由 `verification.code_length` 控制，默认模板为 6 位数字。

#### 响应头

```http
Set-Cookie: refresh_token=<jwt>; HttpOnly; Path=/api/auth; SameSite=Lax
```

#### 响应示例

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "login_mode": "qq",
    "identifier": "123456789",
    "qq_number": "123456789",
    "email": null,
    "role": "user",
    "is_active": true
  }
}
```

> 注意：`refresh_token` 不会出现在 JSON 响应里。

#### 用户字段

| 字段 | 类型 | 说明 |
|------|------|------|
| user.login_mode | string | 登录渠道，`qq` 或 `email` |
| user.identifier | string | 当前登录渠道下的稳定标识，QQ 号或邮箱地址 |
| user.qq_number | string \| null | QQ 登录账号的 QQ 号；邮箱登录时为 `null` |
| user.email | string \| null | 邮箱登录账号的邮箱地址；QQ 登录时为 `null` |
| user.role | string | 用户角色；邮箱登录本轮固定为 `user` |

---

### 刷新会话

**端点**: `POST /api/auth/refresh`  
**认证**: 仅依赖 `refresh_token` HTTPOnly Cookie  
**描述**: 使用 refresh cookie 换取新的 access token，并轮换 refresh cookie

#### 请求体

无请求体。

#### 请求示例

```bash
curl -X POST \
  --cookie "refresh_token=<refresh-jwt>" \
  http://localhost:8000/api/auth/refresh
```

#### 响应头

```http
Set-Cookie: refresh_token=<new-jwt>; HttpOnly; Path=/api/auth; SameSite=Lax
```

#### 响应示例

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "login_mode": "qq",
    "identifier": "123456789",
    "qq_number": "123456789",
    "email": null,
    "role": "user",
    "is_active": true
  }
}
```

---

### 退出登录

**端点**: `POST /api/auth/logout`  
**认证**: 无需 Bearer token；如果存在 refresh cookie，则会被清理  
**描述**: 清除 refresh cookie，结束浏览器会话

#### 响应

- 状态码：`204 No Content`
- 响应头会返回一个已过期的 `refresh_token` cookie

---

### 获取当前用户信息

**端点**: `GET /api/auth/me`  
**认证**: 需要 access token Bearer 认证  
**描述**: 返回当前登录用户的基础信息

#### 请求示例

```bash
curl -H "Authorization: Bearer <access-token>" \
  http://localhost:8000/api/auth/me
```

#### 响应示例

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "login_mode": "qq",
  "identifier": "123456789",
  "qq_number": "123456789",
  "email": null,
  "role": "user",
  "is_active": true
}
```

---

## 房间管理

### 创建房间 ⚠️ 破坏性变更

**端点**: `POST /api/rooms`  
**认证**: 无需认证（建议添加）  
**描述**: 创建新房间记录

**⚠️ 重要变更**: 从v1.1开始，`primary_roompath`字段为**必填**

#### 请求体

```json
{
  "roomid": 123,
  "room_name": "桂林/雁山/05栋/0501",
  "primary_roompath": "桂林/雁山/05栋/0501",
  "threshold": 100.0,
  "electricity_fee": 0.0
}
```

#### 响应示例

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "roomid": 123,
  "room_name": "桂林/雁山/05栋/0501",
  "primary_roompath": "桂林/雁山/05栋/0501",
  "primary_roompath_hash": 1234567890,
  "has_additional_paths": false,
  "is_active": true,
  "source_type": "manual",
  "threshold": 100.0,
  "electricity_fee": 0.0,
  "send_flag": false,
  "created_at": "2025-11-12T10:30:00",
  "updated_at": "2025-11-12T10:30:00"
}
```

---

### 根据roomid查询房间 ⚠️ 破坏性变更

**端点**: `GET /api/rooms/by-roomid/{roomid}`
**认证**: 需要 access token Bearer 认证
**描述**: 根据业务ID查询房间信息。管理员可查询所有房间；普通用户只能查询已绑定房间。

**⚠️ 重要变更**: 从v1.1开始，返回**单个对象**而非数组（roomid现为唯一约束）

#### 响应示例

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "roomid": 123,
  "room_name": "桂林/雁山/05栋/0501",
  "primary_roompath": "桂林/雁山/05栋/0501",
  "has_additional_paths": true,
  "threshold": 100.0,
  "electricity_fee": 85.5
}
```

---

### 查询房间路径树

**端点**: `GET /api/rooms/path-tree?parent={encoded_path}`
**认证**: 需要 access token Bearer 认证
**描述**: 逐层查询可绑定房间路径。叶子节点只返回绑定所需的最小 `roomid`，不会返回电费余额、阈值等房间详情。

#### 响应示例

```json
{
  "children": [
    {
      "name": "0501",
      "is_leaf": true,
      "room_count": 1,
      "roomid": 123
    }
  ],
  "current_level": 3,
  "total_count": 1
}
```

---

### 根据路径查询房间详情

**端点**: `GET /api/rooms/by-path?path={encoded_path}`
**认证**: 需要 access token Bearer 认证
**描述**: 根据完整路径查询房间详情。管理员可查询所有房间；普通用户只能查询已绑定房间。绑定前流程应使用路径树叶子节点中的 `roomid`，不要先读取该详情接口。

#### 响应示例

```json
{
  "roomid": 123,
  "room_name": "0501",
  "electricity_fee": 85.5,
  "threshold": 100.0,
  "primary_roompath": "桂林/雁山/05栋/0501"
}
```

---

## 用户房间绑定

### 创建绑定

**端点**: `POST /api/bindings`
**认证**: 需要 access token Bearer 认证
**描述**: 为当前登录账号创建房间绑定。管理员账号与普通用户账号都可以创建自己的个人绑定；管理员仍可通过详情、通知开关和删除接口管理任意绑定。

#### 请求体

```json
{
  "roomid": 123,
  "notification_enabled": false,
  "binding_proof": "A1B2C3D4E5F6"
}
```

> 普通用户创建绑定必须提供管理员为该房间生成的 `binding_proof`。管理员账号创建自己的个人绑定时可以省略该字段。

#### 响应示例

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "roomid": 123,
  "notification_enabled": false,
  "created_at": "2026-05-05 12:00:00",
  "updated_at": "2026-05-05 12:00:00"
}
```

### 生成房间绑定证明码

**端点**: `GET /api/bindings/proof/{roomid}`
**认证**: 需要管理员 access token Bearer 认证
**描述**: 为指定房间生成一次房间绑定证明码，供管理员通过线下或受控渠道交给真实房间用户。证明码不落库，基于服务端签名密钥和 `roomid` 生成；普通用户不能调用该接口。

#### 响应示例

```json
{
  "roomid": 123,
  "binding_proof": "A1B2C3D4E5F6",
  "proof_version": "v1"
}
```

### 查询当前账号绑定

**端点**: `GET /api/bindings`
**认证**: 需要 access token Bearer 认证
**描述**: 返回当前登录账号自己的绑定列表，并在可用时附带房间信息。管理员账号不会再固定返回空数组；如果管理员创建了个人绑定，也会在这里看到。

---

## 房间同步 🆕

### 手动触发同步

**端点**: `POST /api/rooms/sync`  
**认证**: 无需认证（建议添加）  
**描述**: 手动触发房间数据同步任务

#### 请求体

无需请求体

#### 响应示例 (202 Accepted)

```json
{
  "job_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "status": "pending",
  "message": "同步任务已创建，正在后台执行"
}
```

---

### 查询同步状态

**端点**: `GET /api/rooms/sync/status/{job_id}`  
**认证**: 无需认证  
**描述**: 查询同步任务执行状态

#### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| job_id | UUID | 同步任务ID |

#### 响应示例

**任务运行中**:
```json
{
  "job_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "status": "running",
  "message": "任务运行中（开始于: 2025-11-12 10:30:00）"
}
```

**任务完成**:
```json
{
  "job_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "status": "completed",
  "message": "任务已完成"
}
```

**任务失败**:
```json
{
  "job_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "status": "failed",
  "message": "任务失败: HTTP请求超时"
}
```

---

### 查询同步历史

**端点**: `GET /api/rooms/sync/history`  
**认证**: 无需认证  
**描述**: 查询最近的同步历史记录（默认10条）

#### 响应示例

```json
[
  {
    "id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
    "sync_type": "manual",
    "started_at": "2025-11-12T10:30:00",
    "completed_at": "2025-11-12T10:31:25",
    "status": "completed",
    "stats": {
      "total": 6000,
      "new": 150,
      "updated": 5800,
      "failed": 50,
      "skipped": 0
    },
    "error_message": null
  }
]
```

---

### 查询房间所有路径

**端点**: `GET /api/rooms/{roomid}/paths`  
**认证**: 无需认证  
**描述**: 查询房间的主路径和所有额外路径

#### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| roomid | integer | 房间业务ID |

#### 响应示例

```json
{
  "roomid": 123,
  "primary_roompath": "桂林/雁山/05栋/0501",
  "additional_paths": [
    "桂林/雁山/05栋/05楼/0501",
    "广西/桂林/雁山/05栋/0501"
  ],
  "total_paths": 3
}
```

---

## 错误响应

所有API错误响应遵循统一格式：

```json
{
  "error": "错误类型描述",
  "message": "详细错误信息"
}
```

### HTTP 状态码

| 状态码 | 说明 |
|--------|------|
| 200 | 请求成功 |
| 201 | 资源创建成功 |
| 400 | 请求参数错误 |
| 401 | 未认证或认证失败 |
| 403 | 无权限访问 |
| 404 | 资源未找到 |
| 500 | 服务器内部错误 |
| 503 | 服务不可用（如数据库连接失败） |

---

## 认证头格式

需要 access token 认证的端点需在请求头中包含：

```
Authorization: Bearer <your-jwt-token>
```

示例：
```bash
curl -H "Authorization: Bearer eyJhbGc..." \
  http://localhost:8000/api/protected-endpoint
```

`POST /api/auth/refresh` 不读取 Bearer token，只读取浏览器或客户端自动附带的 `refresh_token` Cookie。

---

## 数据验证

请求数据验证规则：

- **Email**: 必须是有效的邮箱格式
- **Password**: 最少8个字符
- **Username**: 3-50个字符，仅允许字母、数字、下划线

---

## 速率限制

**当前状态**: 未启用  
**计划**: 每用户每分钟100次请求

---

## API 版本控制

**当前版本**: v1.0  
**基础路径**: `/api`  
**计划**: 未来版本将使用 `/api/v2` 路径

---

**文档更新日期**: 2026-04-11  
**API版本**: 1.1  
**重要变更**: 
- v1.1: 登录链路改为 `verify-and-login` + refresh cookie，会话刷新与 logout 走 `/api/auth/refresh`、`/api/auth/logout`
- v1.1: 新增房间同步API（4个端点）
- v1.1: CreateRoomRequest新增必填字段`primary_roompath`
- v1.1: GET /api/rooms/by-roomid返回单对象（非数组）
