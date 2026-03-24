# Maintainability Metrics

## Current metrics

| Metric | Current value | Evidence |
| --- | --- | --- |
| Rust test pass count | 111 passed | `cargo test` |
| Auth integration smoke | 5 passed | `cargo test --test auth_integration_test` |
| Frontend architecture lint | passing | `pnpm lint`, `scripts/check-architecture.ps1` |
| Remaining optimized mainlines | 0 | `docs/architecture/DUAL_IMPLEMENTATION_LEDGER.md` |
| Frontend generated assets tracked by git | 0 | `.gitignore` + `static/.gitkeep` |

## Quality gate owners

| Gate | Owner |
| --- | --- |
| Rust compile / tests | 仓库维护者 |
| Frontend lint / build | 前端维护者 |
| Release manifest / deploy result / smoke | 发布执行人 |
| Architecture script | 仓库维护者 |

## Current gate set

- `cargo test`
- `pnpm lint`
- `pnpm build:prod`
- `powershell -ExecutionPolicy Bypass -File scripts/check-architecture.ps1`

## Notes

- 当前指标用于维护性趋势跟踪，不是 SLA。
- 若新增质量门禁，应同时明确 owner 和失败处理方式。
