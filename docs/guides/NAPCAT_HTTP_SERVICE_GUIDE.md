# NapCat HTTP 机器人服务接入说明

## 当前运行基线
- **当前API端点**: `http://example.invalid:3000/send_private_msg`
- **认证方式**: Bearer Token
- **当前token管理方式**: 运行时通过 `qq_bot.bearer_token` 或 `APP__QQ_BOT__BEARER_TOKEN(_FILE)` 注入，文档不再明文记录 token
- **最近联通性验证**: 2026-03-30 向 `100000001` 发送字符串消息 `"你好"`，返回 `status=ok`、`retcode=0`、`message_id=992143794`

## 历史测试概述
以下错误处理和边界条件测试记录形成于 2025-11-12，保留其行为观察结论；当前运行真源以上面的最新 API 端点和 token 管理方式为准。

## 历史测试环境
- **测试时间**: 2025-11-12
- **认证方式**: Bearer Token
- **说明**: 历史测试所用 token 已轮换失效，当前请以运行时配置或 secret file 为准

### 认证信息说明
- Bearer Token 用于 API 身份验证
- 请求头格式为 `Authorization: Bearer <token>`
- 令牌验证失败时 API 会返回 HTTP 403 状态码
- 当前 token 已轮换，避免在日志、文档或公共代码库中暴露

## 测试用例与结果

### 1. 正常情况测试 ✅
**测试场景**: 使用正确的参数发送消息
```json
{
  "user_id": "100000001",
  "message": [
    {
      "type": "text",
      "data": {
        "text": "我是机器人"
      }
    }
  ]
}
```

**响应结果**:
- HTTP状态码: `200`
- 响应状态: `成功`
- **完整API响应**:
```json
{
  "status": "ok",
  "retcode": 0,
  "data": {
    "message_id": 583595608
  },
  "message": "",
  "wording": "",
  "echo": "vqy9m0vpr8",
  "stream": "normal-action"
}
```

**分析**: API正常工作，成功返回消息ID。响应包含完整的元数据字段。

---

### 2. 错误Bearer Token测试 🔒
**测试场景**: 使用无效的认证令牌
```
Authorization: Bearer wrong_token_12345
```

**响应结果**:
- HTTP状态码: `403`
- 响应状态: `认证失败`
- **完整API响应**:
```json
{
  "message": "token verify failed!"
}
```

**分析**: API正确处理了无效token，返回403状态码和明确的错误信息。响应格式相对简单，只包含错误消息。

---

### 3. 错误User ID测试 👤
**测试场景**: 使用不存在的用户ID `9999999999`

**响应结果**:
- HTTP状态码: `200`
- 响应状态: `业务逻辑失败`
- **完整API响应**:
```json
{
  "status": "failed",
  "retcode": 200,
  "data": null,
  "message": "无法获取用户信息",
  "wording": "无法获取用户信息",
  "echo": "2vahxfof4wo",
  "stream": "normal-action"
}
```

**分析**: API能够正确识别不存在的用户，虽然HTTP状态码是200，但通过`status: "failed"`和`retcode: 200`表示业务逻辑失败。响应保持了完整的标准格式。

---

### 4. 错误Message格式测试 📝

#### 4.1 缺少message字段
**请求**: 完全省略message字段
**响应结果**:
- HTTP状态码: `200`
- **完整API响应**:
```json
{
  "status": "failed",
  "retcode": 200,
  "data": null,
  "message": "Cannot read properties of undefined (reading 'type')",
  "wording": "Cannot read properties of undefined (reading 'type')",
  "echo": "nf073yl1db",
  "stream": "normal-action"
}
```
**分析**: API尝试处理undefined的message对象，导致JavaScript风格的错误。

#### 4.2 message不是数组
**请求**: `message: "这是一个字符串而不是数组"`
**响应结果**:
- HTTP状态码: `200`
- **完整API响应**:
```json
{
  "status": "ok",
  "retcode": 0,
  "data": {
    "message_id": 32231856
  },
  "message": "",
  "wording": "",
  "echo": "15vzkpv5ubt",
  "stream": "normal-action"
}
```
**分析**: API对此进行了容错处理，成功发送消息。可能内部将字符串转换为了有效的消息格式。

#### 4.3 缺少type字段
**请求**: message数组中对象缺少type字段
**响应结果**:
- HTTP状态码: `200`
- **完整API响应**:
```json
{
  "status": "failed",
  "retcode": 200,
  "data": null,
  "message": "未知的消息类型：undefined",
  "wording": "未知的消息类型：undefined",
  "echo": "amcaux2qkce",
  "stream": "normal-action"
}
```
**分析**: API严格验证消息类型，undefined类型被明确拒绝。

#### 4.4 缺少data字段
**请求**: message对象中缺少data字段
**响应结果**:
- HTTP状态码: `200`
- **完整API响应**:
```json
{
  "status": "failed",
  "retcode": 200,
  "data": null,
  "message": "Cannot read properties of undefined (reading 'text')",
  "wording": "Cannot read properties of undefined (reading 'text')",
  "echo": "hy8f2cydx2",
  "stream": "normal-action"
}
```
**分析**: API尝试访问undefined的data对象，导致JavaScript风格的错误。

#### 4.5 缺少text字段
**请求**: data中缺少text字段
**响应结果**:
- HTTP状态码: `200`
- **完整API响应**:
```json
{
  "status": "ok",
  "retcode": 0,
  "data": {
    "message_id": 794398855
  },
  "message": "",
  "wording": "",
  "echo": "gorvmcsq5ab",
  "stream": "normal-action"
}
```
**分析**: API对此进行了容错处理，成功发送消息。可能使用了默认的文本内容或空消息。

---

## API行为分析

### 错误处理机制
1. **认证错误**: 返回HTTP 403状态码，明确的token验证失败信息
2. **业务逻辑错误**: 返回HTTP 200，但通过`status: "failed"`和`retcode: 200`表示失败
3. **参数验证错误**: 返回HTTP 200，错误信息在message字段中

### 响应格式一致性
所有响应都遵循统一的JSON格式，但根据不同情况有所差异：

#### 成功响应格式：
```json
{
  "status": "ok",
  "retcode": 0,
  "data": {
    "message_id": "唯一消息标识符"
  },
  "message": "",
  "wording": "",
  "echo": "请求唯一标识符",
  "stream": "normal-action"
}
```

#### 业务失败响应格式：
```json
{
  "status": "failed",
  "retcode": 200,
  "data": null,
  "message": "具体错误信息",
  "wording": "同message字段",
  "echo": "请求唯一标识符",
  "stream": "normal-action"
}
```

#### 认证失败响应格式：
```json
{
  "message": "token verify failed!"
}
```

**字段说明**：
- `status`: "ok"表示成功，"failed"表示业务逻辑失败
- `retcode`: 0表示成功，200表示业务失败
- `data`: 成功时包含消息ID，失败时为null
- `message`: 错误信息或空字符串
- `wording`: 与message相同，用于显示
- `echo`: 每个请求的唯一标识符
- `stream`: 固定为"normal-action"

### 安全性评估
- ✅ 认证机制有效，能正确拒绝无效token
- ✅ 用户验证机制有效，能识别不存在的用户
- ⚠️ 参数验证较为宽松，某些格式错误仍能成功处理

### 建议改进
1. **统一错误状态码**: 建议业务逻辑错误也使用适当的HTTP状态码(如400, 404)
2. **加强参数验证**: 对message格式进行更严格的验证
3. **错误信息国际化**: 考虑提供英文错误信息

## 测试结论
该QQ机器人API整体功能正常，认证和用户验证机制工作良好。

### 主要发现：
1. **认证机制**: 严格有效，正确返回403状态码
2. **用户验证**: 能准确识别不存在的用户，使用业务失败状态
3. **参数处理**: 具有一定的容错能力，但验证逻辑不够严格
4. **响应格式**: 整体一致，但认证失败时格式简化

### API特点：
- 使用JavaScript风格的错误信息
- 每个请求都有唯一的echo标识符
- 支持一定程度的参数容错处理
- HTTP状态码与业务状态分离的设计模式

### 改进空间：
- HTTP状态码语义化改进
- 参数验证严格性提升
- 错误响应格式统一化
- 错误信息国际化支持

---
**测试执行时间**: 2025-11-12 16:48:00 UTC+8
