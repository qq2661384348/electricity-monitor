# API 参考文档

## 概述

Electricity Monitor Backend REST API v1.0

**基础URL**: `http://localhost:8000/api` (开发环境)  
**认证方式**: JWT Bearer Token  
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

**成功响应** (200 OK):
```json
{
  "status": "ok",
  "database": "connected",
  "message": "Service and database are healthy"
}
```

**失败响应** (503 Service Unavailable):
```json
{
  "error": "数据库操作失败",
  "message": "Database error: connection refused"
}
```

---

## 认证 (待实现)

### 用户登录

**端点**: `POST /api/auth/login`  
**认证**: 无需认证  
**描述**: 用户登录并获取JWT token

#### 请求体

```json
{
  "email": "user@example.com",
  "password": "password123"
}
```

#### 响应示例

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 86400
}
```

---

### 用户注册

**端点**: `POST /api/auth/register`  
**认证**: 无需认证  
**描述**: 注册新用户

#### 请求体

```json
{
  "username": "newuser",
  "email": "user@example.com",
  "password": "password123"
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

需要认证的端点需在请求头中包含：

```
Authorization: Bearer <your-jwt-token>
```

示例：
```bash
curl -H "Authorization: Bearer eyJhbGc..." \
  http://localhost:8000/api/protected-endpoint
```

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

**文档更新日期**: 2025-10-21  
**API版本**: 1.0
