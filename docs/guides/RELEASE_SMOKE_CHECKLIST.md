# 发布冒烟检查清单

用于 release artifact 解压后的最小验收闭环。

## 前置条件

- 已下载并解压 `release-<tag>.tar.gz`
- 已填写 `.env`
- `release-manifest.json` 存在
- `deploy.sh` 已执行完成

## 冒烟步骤

1. 确认 `release-manifest.json` 中的 `git_tag`、`git_sha`、`app_image_ref` 可读。
2. 确认 `deploy-result.json` 已生成，且 `status` 为 `deployed`。
3. 访问 `GET /api/health`，预期 `200`。
4. 访问 `GET /api/health/db`，预期 `200`。
5. 使用受保护接口做一次鉴权验证，例如 `GET /api/auth/me`。
6. 确认前端静态资源入口可访问。
7. 记录本次验收人、时间、版本、结果。

## 示例命令

```bash
curl -f http://127.0.0.1:11450/api/health
curl -f http://127.0.0.1:11450/api/health/db
./smoke.sh
cat release-manifest.json
cat deploy-result.json
```

## 回滚触发条件

- `/api/health` 或 `/api/health/db` 失败
- 关键受保护接口鉴权异常
- 静态资源入口不可访问

满足任一条件时，按 `deploy.sh` 记录的上一个备份版本执行回滚。
