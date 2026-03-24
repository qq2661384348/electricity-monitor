# Electricity Monitor 仓库记忆：第二轮架构升级关注点

## 后端维护性热点
- `src/infrastructure/repositories/room_repository.rs` 规模过大，混合了查询、更新、同步日志、缓存支持、恢复时间持久化等多类职责。
- `src/domain/services/notification_gate.rs` 与 `src/domain/services/notification_service.rs` 体量较大，说明通知域内部边界仍不够收敛。
- `src/handlers/binding.rs` 与 `src/handlers/auth.rs` 仍有直接依赖 repository 的旧入口，需要继续往 application/usecase 层收敛。

## 重复/过渡态实现
- 同时存在优化版与非优化版服务：
  - 当前已收敛删除 `electricity_service_optimized.rs` 与 `room_sync/sync_service_optimized.rs`
- 这类平行文件会增加认知成本、修改重复成本和未来行为漂移风险。

## 前端维护性热点
- `DashboardPage.tsx` 是高耦合页面容器：聚合认证判断、查询结果转换、事件处理、多个 modal 状态。
- `useDashboardData.ts` 中仍保留 deprecated hook，说明前端数据层还处于迁移中。
- `frontend/src/services/api.ts` 集中定义多类接口，适合作为后续按 feature 拆分的候选点。
- `authStore.ts`、React Query、Axios 已形成“本地状态 + 服务端状态 + HTTP 封装”三层雏形，后续升级应以它为基础收敛，而不是重造状态体系。
- `BindRoomModal.tsx` 已经表现出“流程型组件”特征，适合进一步拆成 step state / query / presentation。

## 文档与基线一致性问题
- `docs/guides/TECHNICAL_DEBT.md` 的测试数量和完成度描述已明显落后于当前仓库状态。
- `docs/INDEX.md` 仍保留旧的“待创建”描述。
- 后续做架构升级前，应先判断哪些文档能继续当真源，哪些只能作为历史参考。

## 第二轮方案设计建议输入
- 优先做“边界收敛”而不是只做风格重写。
- 重点关注：
  - 仓储职责拆分
  - 通知域边界收敛
  - binding/auth handler 下沉
  - 前端页面容器瘦身与 feature 分层
  - 文档真源和发布真源统一
