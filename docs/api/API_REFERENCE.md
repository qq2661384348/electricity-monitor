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

## 认证

当前认证链路采用“双 token”模型：

- access token：通过 JSON 响应返回，由前端只保存在内存中，并以 `Authorization: Bearer <token>` 调用受保护接口
- refresh token：只通过 HTTPOnly Cookie `refresh_token` 下发，不出现在 JSON 响应里

### 发送验证码

**端点**: `POST /api/auth/send-verification-code`  
**认证**: 无需认证  
**描述**: 向指定 QQ 发送一次性验证码

#### 请求体

```json
{
  "qq_number": "123456789",
  "captcha_token": "optional-captcha-token"
}
```

#### 响应示例

```json
{
  "message": "验证码已发送",
  "qq_number": "123456789"
}
```

---

### 验证验证码并登录

**端点**: `POST /api/auth/verify-and-login`  
**认证**: 无需认证  
**描述**: 校验 QQ 验证码，创建或查找用户，并签发 access token 与 refresh cookie

#### 请求体

```json
{
  "qq_number": "123456789",
  "code": "123456"
}
```

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
    "qq_number": "123456789",
    "role": "user",
    "is_active": true
  }
}
```

> 注意：`refresh_token` 不会出现在 JSON 响应里。

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
    "qq_number": "123456789",
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
  "qq_number": "123456789",
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
**认证**: 无需认证  
**描述**: 根据业务ID查询房间信息

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
