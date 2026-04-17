---
type: decision
status: active
scope: 前端工具链与工作区协作
created_at: 2026-04-17
updated_at: 2026-04-17
sources:
  - frontend/package.json
  - frontend/AGENTS.md
  - .github/workflows/ci.yml
summary: frontend 工作区固定以 bun 作为唯一包管理与锁文件真源
superseded_by:
---

# 前端包管理固定为 Bun

## 背景

- 前端曾同时承载过 `pnpm` / `npm` / `bun` 的迁移痕迹，容易导致 lockfile、CI 和文档真源漂移。
- 当前前端工作区已经切到 Bun，并以 `bun.lock`、`frontend/package.json` 和对应 CI 命令为唯一真源。

## 目标

- 保持前端包管理入口单一。
- 避免多套 lockfile 和多套命令并存。
- 让传递依赖安全修复通过 Bun 的解析树和 `overrides` 稳定落地。

## 候选方案

### 方案 A

- 保留 Bun 作为唯一包管理真源。

### 方案 B

- 同时保留 `npm` / `pnpm` 兼容路径。

## 最终选择

- 选择方案 A，前端工作区只使用 Bun。

## 选择理由

- 单一 lockfile 和单一命令链最容易保持 CI、文档和本地开发一致。
- `frontend/package.json` 中的 `overrides` 已成为前端传递依赖安全修复真源，双轨维护会增加漂移风险。

## 后果与影响

- 前端依赖安装、测试、lint、构建、bundle 预算和审计都统一使用 Bun 命令。
- 不再接受重新引入 `pnpm-lock.yaml` 或 `package-lock.json` 作为有效真源。

## 关联长期记忆

- `../long-term/semantic/frontend-architecture.md`

## 后续动作

- [ ] 若后续升级工具链，继续同步复核 `bun.lock`、`overrides`、CI 和前端文档。
