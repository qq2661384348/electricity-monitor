# 机密项清单

## 当前真源

- 生产敏感配置通过 Compose secrets 提供。
- 应用通过 `APP__*__*_FILE` 读取 `/run/secrets/*` 中的内容。
- release 包中的 `.env.example` 只记录 secret file 路径，不再记录秘密原文。

## 清单

| Secret | 负责人 | 来源 | 轮换方式 | 暴露面 |
| --- | --- | --- | --- | --- |
| Database password | 运维 / 部署执行人 | `APP_DATABASE_PASSWORD_SECRET_FILE` -> `/run/secrets/app_database_password` | 数据库密码变更时同步轮换 | app 容器运行时 |
| JWT secret | 运维 / 部署执行人 | `APP_JWT_SECRET_SECRET_FILE` -> `/run/secrets/app_jwt_secret` | JWT 密钥轮换时同步失效旧 token | app 容器运行时 |
| NapCat HTTP service bearer token | 机器人接口 owner / 运维 | `APP_QQ_BOT_BEARER_TOKEN_SECRET_FILE` -> `/run/secrets/app_qq_bot_bearer_token` | 机器人 token 更新时轮换 | app 容器运行时 |

## 规则

- 不在 `config/production.toml` 中保留真实秘密。
- 不在 `.env` 中存秘密原文，只保留 secret file 路径。
- 缺失 secret file 时，production 启动必须 fail-fast。
- 发布日志与部署记录不得打印 secret 原文。
