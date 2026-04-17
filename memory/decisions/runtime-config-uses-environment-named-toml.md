---
type: decision
status: active
scope: 运行时配置与环境语义
created_at: 2026-04-17
updated_at: 2026-04-17
sources:
  - src/config/app.rs
  - config/development.toml.example
  - config/production.toml.example
summary: config 目录只保留与 APP_ENV 对应的单一活动运行时 toml 文件
superseded_by:
---

# 运行时配置使用按环境命名的单一 TOML

## 背景

- 旧的 `default.toml` 语义容易与模板、默认值和活动运行时配置混淆。
- 当前配置加载已经明确要求使用 `development.toml` 或 `production.toml`，并与 `APP_ENV` 对齐。

## 目标

- 让运行时配置文件名直接表达环境语义。
- 保证 `config/` 下只有一个活动 `.toml`，避免双文件并存造成误读。
- 让开发和生产模板路径稳定可预期。

## 候选方案

### 方案 A

- 使用按环境命名的活动运行时文件，并要求 `config/` 下只保留一个活动 `.toml`。

### 方案 B

- 保留模糊命名的默认文件或多活动文件并存。

## 最终选择

- 选择方案 A，运行时配置改为按环境命名且只保留单一活动 `.toml`。

## 选择理由

- 环境语义直接写进文件名，最容易与 `APP_ENV`、模板复制流程和 fail-fast 校验保持一致。
- 单一活动文件能减少发布、测试和本地联调时的配置歧义。

## 后果与影响

- 开发环境从 `config/development.toml.example` 复制为 `config/development.toml`。
- 生产环境从 `config/production.toml.example` 复制为 `config/production.toml`。
- `config/` 下出现多个活动 `.toml` 或文件名与 `APP_ENV` 不一致时，配置加载会明确失败。

## 关联长期记忆

- `../long-term/semantic/config-and-environments.md`

## 后续动作

- [ ] 后续若调整运行时配置链路，同步复核模板、脚本、CI 和部署文档中的文件名约定。
