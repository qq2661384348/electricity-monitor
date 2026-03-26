# Infra Tests

`tests/infra/` 预留给需要显式依赖 PostgreSQL、Redis、外部服务或 service container 的环境型测试。

当前仓库的环境型覆盖仍主要位于源码内 `#[cfg(test)]` 测试，例如：

- `src/infrastructure/database/pool.rs`
- `src/infrastructure/redis/pool.rs`
- `src/domain/services/rate_limiter.rs`
- `src/infrastructure/repositories/room_repository.rs`
- `src/domain/services/room_sync/crawler/client.rs`

这些测试通过 `RUN_INTEGRATION_TESTS=1` 或 Redis 连接变量启用；后续新增重型 infra 回归时，优先在这里落独立 test target，而不是继续把环境门槛散落到默认源码测试里。

外部网络测试不再与这批本地 infra 测试共用开关。当前外部网络测试统一改为 `RUN_EXTERNAL_INTEGRATION_TESTS=1` 显式启用，避免默认 CI 因公网依赖失真。
