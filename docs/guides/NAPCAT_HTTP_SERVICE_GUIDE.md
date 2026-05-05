# NapCat HTTP 机器人服务接入说明

本仓库通过 NapCat 提供的 HTTP action 接口发送验证码和通知。公开仓库只保留通用接入方式，不记录任何真实账号、Bearer token、生产地址或联通性回执。

## 接入前提

1. 在目标环境部署 NapCat，并完成登录。
2. 启用 NapCat 的 HTTP 服务能力。
3. 为机器人 HTTP 服务准备 Bearer token，并通过运行时环境变量或 secret file 注入应用。

## 推荐配置

开发或联调环境必须把 `qq_bot.api_url` 指向当前可访问的 NapCat action endpoint。配置模板只保留中文占位和示例注释，不内置本地 mock 地址或任何部署环境的真实值：

```toml
[qq_bot]
# QQ 私聊发送链路：登录验证码和电量通知都会通过此 NapCat action 发送。
# NapCat HTTP action 地址，例如：http://127.0.0.1:3000/send_private_msg
api_url = "你的napcat应用URL"
# 必须由部署者手动填写，前端会通过 /api/public-config 公开展示该值。
public_qq_number = ""
bearer_token = ""
timeout_seconds = 10
```

复制到运行时配置后，`qq_bot.api_url` 应填写为 `http(s)://<napcat-host>:<port>/send_private_msg`，`qq_bot.bearer_token` 应通过运行时配置或 `APP__QQ_BOT__BEARER_TOKEN_FILE` 注入真实 token。验证码发送和电量预警通知都会复用这一段 `[qq_bot]` 配置。

`qq_bot.public_qq_number` 是用户添加好友时看到的机器人 QQ 号真源。它必须手动配置，不能从管理员 QQ 推断，也不能通过自动读取 NapCat 登录信息来替代。

生产环境不要把 token 写回仓库。推荐通过以下任一方式注入：

- `APP__QQ_BOT__BEARER_TOKEN`
- `APP__QQ_BOT__BEARER_TOKEN_FILE`

如果部署地址不是默认的 loopback endpoint，也通过 `APP__QQ_BOT__API_URL` 覆盖。
如果机器人 QQ 号变更，通过运行时 TOML 或 `APP__QQ_BOT__PUBLIC_QQ_NUMBER` 同步，确保 `/api/public-config` 和前端公告同时更新。

## 验证建议

- 自动化回归使用 `tests/contracts/send_verification_code_integration_test.rs` 的本地 mock HTTP 服务。
- 真实联通性验证应在私有运维环境执行，不要把机器人账号、token、消息回执或聊天截图写回公开仓库文本。
- 出现 `USER_NOT_FRIEND` 时，前端应提示用户先添加 `qq_bot.public_qq_number` 配置的机器人 QQ；如果仍无法收到验证码，再引导联系 `admin.default_qq_number` 配置的管理员 QQ。

## 开源仓库约束

- 不在已纳入版本控制的配置模板、`docs/`、`memory/`、测试夹具或示例命令中写入真实机器人账号。
- 不在提交历史中保留真实 token、账号、生产地址或运维回执。
- 若机器人账号、部署地址或 token 发生变更，只更新私有运行时配置和运维渠道，不更新公开仓库中的真实值。
