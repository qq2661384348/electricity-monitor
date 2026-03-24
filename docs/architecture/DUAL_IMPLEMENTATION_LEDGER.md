# Dual Implementation Ledger

本台账只记录当前仓库里已经存在的平行实现，以及当前仓库证据能确认的主线。

## Current inventory

| 域 | 当前主线 | 平行实现 | 仓库证据 | 删除条件状态 |
| --- | --- | --- | --- | --- |
| Electricity | `electricity_service.rs` | `electricity_service_optimized.rs` | `src/bootstrap/runtime.rs` 当前实例化 `ElectricityService::new(...)` | Removed |
| Room Sync | `room_sync/sync_service.rs` | `room_sync/sync_service_optimized.rs` | `src/bootstrap/runtime.rs` 与 `src/handlers/room_sync.rs` 当前实例化 `RoomSyncService::new(...)` | Removed |

## Guardrails

- 新代码不得新增新的同域双实现主线。
- 在删除条件未明确前，禁止把新需求继续写入平行实现。
- 收敛时必须补一份回归清单，至少覆盖当前已被调用的主线路径。

## Remaining pending decisions

- 当前双实现台账没有剩余待收敛项；后续若新增平行实现，必须先登记主线与删除条件。

## Defined deletion conditions

### Room Sync optimized

- 仓库内不再有任何 `sync_service_optimized` 引用。
- `cargo test` 通过。
- 手动同步入口 `POST /api/rooms/sync` 的行为不需要回滚到 optimized 版本。
- 删除后，`RoomSyncService::new(...)` 仍是唯一被 runtime / handler 调用的主线。
