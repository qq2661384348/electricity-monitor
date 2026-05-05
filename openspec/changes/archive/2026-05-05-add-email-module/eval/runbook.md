# Eval Runbook

## 准备

1. 确认不需要真实 SMTP password。
2. 确认工作树中没有 tracked 文件包含用户提供的 SMTP 授权码原文。
3. 若执行 release compose config，需要准备临时 `.env` 和空 secret 文件；自动门禁可先做路径和语法检查。

## 执行

```bash
bash openspec/changes/add-email-module/eval/commands.sh
```

## 读取结果

- `cargo test --lib` 通过表示配置和邮件模块单元断言通过。
- `cargo test --test release_readiness_test` 通过表示 release readiness 契约未因部署链路变更漂移。
- `cargo clippy --all-targets -- -D warnings` 通过表示新增代码没有 clippy warning。
- `cargo audit -q` 通过表示 Rust 依赖审计当前无阻断项。
- `bash -n deploy/deploy.sh` 通过表示部署脚本语法没有破坏。
- `docker compose -f deploy/docker-compose.local.yml config` 通过表示本地 Docker compose 模板可解析。

## 常见失败模式

- `lettre` feature 编译失败：回到 `Cargo.toml` 调整 feature，优先保持 rustls async。
- production secret 校验测试失败：检查 `email.smtp_password_file` 解析和 `validate_sensitive_config`。
- secret 扫描命中：确认没有把 SMTP 授权码原文写入模板、docs、memory 或 spec。
