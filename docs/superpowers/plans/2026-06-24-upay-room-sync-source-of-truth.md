# Upay Room Sync Source Of Truth Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 Upay 实时房间树成为房间数据源真相，后端定时自动同步新增、路径变化和源端缺失房间停用，并以可回滚方式修正生产数据库。

**Architecture:** `RoomSyncService` 继续作为唯一写库入口，新增“源端缺失则停用”的差异计划和停用保护阈值；`RoomSyncUseCase` 统一手动与定时同步任务，任务完成后刷新电费缓存和内存 `RoomPathTree`。生产修复优先通过发布后的同步接口触发应用逻辑，数据库直连只用于备份、只读核验和应急回滚。

**Tech Stack:** Rust、Axum、Diesel async、PostgreSQL、Tokio 后台任务、GitHub Actions、现有 `deploy/` 离线部署链路。

---

## Files

- Modify: `src/config/room_sync.rs`，增加停用保护配置。
- Modify: `config/development.toml.example`，记录新配置默认值。
- Modify: `config/production.toml.example`，记录新配置默认值。
- Modify: `src/domain/services/room_sync/sync_service.rs`，实现新增、更新、停用统一差异计划和统计。
- Modify: `src/modules/room_sync/application/mod.rs`，复用手动同步任务逻辑并支持定时同步触发。
- Modify: `src/bootstrap/runtime.rs`，启动周期性房间同步后台任务。
- Modify: `src/bootstrap/app.rs`，在状态初始化后挂载周期性同步任务。
- Modify: `memory/long-term/semantic/config-and-environments.md`，同步运行时配置事实。
- Modify: `memory/long-term/semantic/backend-seams.md`，同步后端同步接缝事实。
- Optional Modify: `docs/guides/TESTING.md`，若新增测试入口需要对外记录。

## Task 1: Sync Policy Tests

**Files:**
- Modify: `src/domain/services/room_sync/sync_service.rs`

- [ ] **Step 1: Add failing unit tests for stale-room planning**

Add tests in the existing `#[cfg(test)]` module:

```rust
#[test]
fn sync_plan_marks_active_rooms_missing_from_source_as_stale() {
    let existing = vec![
        test_room(1, "校区/楼/101", true),
        test_room(2, "校区/楼/102", true),
    ];
    let latest = vec![RoomData::new(1, vec!["校区/楼/101".to_string()])];

    let plan = SyncPlan::from_latest_and_existing(latest, existing);

    assert_eq!(plan.active_roomids, vec![1]);
    assert_eq!(plan.stale_roomids, vec![2]);
}
```

- [ ] **Step 2: Run the target test and confirm it fails**

Run:

```bash
cargo test --lib domain::services::room_sync::sync_service::tests::sync_plan_marks_active_rooms_missing_from_source_as_stale -- --nocapture
```

Expected: FAIL because `SyncPlan` does not exist yet.

- [ ] **Step 3: Add failing unit tests for deactivation guard**

Add tests:

```rust
#[test]
fn deactivation_guard_rejects_source_count_below_minimum_for_existing_database() {
    let err = validate_deactivation_plan(5_594, 100, 100, 0.5, 1_000)
        .expect_err("低于最小源端数量时必须拒绝停用");

    assert!(err.to_string().contains("源端房间数量过低"));
}

#[test]
fn deactivation_guard_allows_current_production_scale_delta() {
    validate_deactivation_plan(5_594, 4_622, 2_068, 0.5, 1_000)
        .expect("当前生产修正规模应在默认停用保护阈值内");
}

#[test]
fn deactivation_guard_rejects_excessive_stale_ratio() {
    let err = validate_deactivation_plan(5_594, 2_000, 4_000, 0.5, 1_000)
        .expect_err("超过停用比例上限时必须拒绝停用");

    assert!(err.to_string().contains("停用比例"));
}
```

- [ ] **Step 4: Run the target tests and confirm they fail**

Run:

```bash
cargo test --lib domain::services::room_sync::sync_service::tests::deactivation_guard -- --nocapture
```

Expected: FAIL because guard functions do not exist yet.

## Task 2: Sync Implementation

**Files:**
- Modify: `src/config/room_sync.rs`
- Modify: `config/development.toml.example`
- Modify: `config/production.toml.example`
- Modify: `src/domain/services/room_sync/sync_service.rs`

- [ ] **Step 1: Add config fields**

Add to `RoomSyncConfig`:

```rust
#[serde(default = "default_max_deactivate_ratio")]
pub max_deactivate_ratio: f64,

#[serde(default = "default_min_source_room_count")]
pub min_source_room_count: usize,
```

Defaults:

```rust
fn default_max_deactivate_ratio() -> f64 {
    0.5
}

fn default_min_source_room_count() -> usize {
    1000
}
```

- [ ] **Step 2: Extend `RoomSyncService::new`**

Pass `max_deactivate_ratio` and `min_source_room_count` into `RoomSyncService` from startup and manual sync wiring.

- [ ] **Step 3: Implement pure sync plan and guard**

Create private `SyncPlan` and `validate_deactivation_plan` helpers in `sync_service.rs`. The plan must compute:

```rust
to_create: Vec<RoomData>
to_update: Vec<RoomData>
stale_roomids: Vec<i64>
active_roomids: Vec<i64>
skipped: usize
```

- [ ] **Step 4: Wire stale deactivation into `sync()` before stats completion**

After fetch and before any write, validate stale deactivation. Then perform creates, updates and:

```rust
let deactivated = self.repository.deactivate_except(&plan.active_roomids).await?;
stats.deactivated = deactivated;
```

After any mutation, call:

```rust
self.cache.full_refresh().await?;
```

This refresh is intentionally simple and safe after bulk mutation.

- [ ] **Step 5: Run unit tests**

Run:

```bash
cargo test --lib domain::services::room_sync::sync_service -- --nocapture
```

Expected: all sync service unit tests pass.

## Task 3: Periodic Sync

**Files:**
- Modify: `src/modules/room_sync/application/mod.rs`
- Modify: `src/bootstrap/runtime.rs`
- Modify: `src/bootstrap/app.rs`

- [ ] **Step 1: Add a scheduled sync trigger in `RoomSyncUseCase`**

Add a public method:

```rust
pub async fn trigger_scheduled_sync(&self) -> Result<Option<Uuid>>
```

It must reuse the same running-job registry as manual sync and return `Ok(None)` when another sync is already running.

- [ ] **Step 2: Share the job runner**

Refactor the existing manual task body into a shared runner that accepts `sync_type` and `task_name`, so manual sync writes `sync_type="manual"` and scheduled sync writes `sync_type="scheduled"`.

- [ ] **Step 3: Add runtime scheduler**

Add `spawn_room_sync_scheduler(state: AppState)` in `src/bootstrap/runtime.rs`. It must:

```rust
if !config.room_sync.enabled { return; }
tokio::spawn(async move {
    let delay = Duration::from_secs(config.room_sync.interval_hours * 3600);
    loop {
        tokio::time::sleep(delay).await;
        RoomSyncUseCase::from_state(&state).trigger_scheduled_sync().await
    }
});
```

- [ ] **Step 4: Start scheduler after `AppState` exists**

Call `runtime::spawn_room_sync_scheduler(state.clone())` in `src/bootstrap/app.rs` after path tree initialization.

- [ ] **Step 5: Run compile-focused tests**

Run:

```bash
cargo test --lib bootstrap::runtime modules::room_sync::application -- --nocapture
```

Expected: tests compile and existing tests pass.

## Task 4: Documentation And Memory

**Files:**
- Modify: `memory/long-term/semantic/config-and-environments.md`
- Modify: `memory/long-term/semantic/backend-seams.md`

- [ ] **Step 1: Update config memory**

Record that room sync has safety settings:

```markdown
- `[room_sync]` 包含 `max_deactivate_ratio` 与 `min_source_room_count`，用于阻止外部房间树异常缩小时误停用大量生产房间。
```

- [ ] **Step 2: Update backend seams memory**

Record that room sync now treats Upay as source of truth:

```markdown
- 房间同步以 Upay 实时房间树为源真相；定时同步和管理员手动同步共用同一任务执行器，都会在完成后刷新电费缓存与内存路径树。
```

## Task 5: Verification

**Files:**
- No source edits.

- [ ] **Step 1: Run targeted Rust tests**

Run:

```bash
cargo test --lib domain::services::room_sync::sync_service -- --nocapture
```

- [ ] **Step 2: Run release readiness tests**

Run:

```bash
cargo test --test release_readiness_test
```

- [ ] **Step 3: Run clippy**

Run:

```bash
cargo clippy --all-targets -- -D warnings
```

## Task 6: GitHub Release

**Files:**
- Git state only.

- [ ] **Step 1: Inspect diff**

Run:

```bash
git status -sb
git diff --stat
git diff
```

- [ ] **Step 2: Commit**

Run:

```bash
git add src/config/room_sync.rs config/development.toml.example config/production.toml.example src/domain/services/room_sync/sync_service.rs src/modules/room_sync/application/mod.rs src/bootstrap/runtime.rs src/bootstrap/app.rs memory/long-term/semantic/config-and-environments.md memory/long-term/semantic/backend-seams.md docs/superpowers/plans/2026-06-24-upay-room-sync-source-of-truth.md
git commit -m "Fix room sync source-of-truth updates"
```

- [ ] **Step 3: Push and watch GitHub Actions**

Run:

```bash
git push origin master
gh run list --limit 5
gh run watch <run-id> --exit-status
```

## Task 7: Production Deployment And Data Repair

**Files:**
- Production host only.

- [ ] **Step 1: Create production database backup before writes**

Run on server through `ssh ali`:

```bash
cd /rust-project/electricity-monitor
mkdir -p backups
docker compose exec -T postgres pg_dump -U "$POSTGRES_USER" "$POSTGRES_DB" > "backups/rooms-sync-before-$(date +%Y%m%d-%H%M%S).sql"
```

- [ ] **Step 2: Read-only preflight counts**

Run read-only SQL:

```sql
SELECT COUNT(*) FROM rooms WHERE is_active = TRUE;
SELECT COUNT(*) FROM room_paths;
SELECT COUNT(*) FROM rooms WHERE primary_roompath LIKE '文昌校区/%' AND is_active = TRUE;
SELECT COUNT(*) FROM rooms WHERE primary_roompath LIKE '商业、教学、教职工用电/%' AND is_active = TRUE;
```

- [ ] **Step 3: Deploy release artifact using existing offline deployment flow**

Use the repository’s existing GitHub Actions release artifact and `deploy/` process. Because the server cannot reach Docker registries directly, upload the built app artifact from the local environment to `/rust-project/electricity-monitor` and run the existing deploy script there.

- [ ] **Step 4: Trigger admin sync through application API**

After deployment, call:

```bash
curl -X POST "$BASE_URL/api/rooms/sync" -H "Authorization: Bearer $ADMIN_ACCESS_TOKEN"
```

Then poll:

```bash
curl "$BASE_URL/api/rooms/sync/status/$JOB_ID" -H "Authorization: Bearer $ADMIN_ACCESS_TOKEN"
```

- [ ] **Step 5: Post-sync verification**

Verify:

```sql
SELECT COUNT(*) FROM rooms WHERE is_active = TRUE;
SELECT COUNT(*) FROM room_paths;
SELECT COUNT(*) FROM rooms WHERE primary_roompath LIKE '文昌校区/%' AND is_active = TRUE;
SELECT COUNT(*) FROM rooms WHERE primary_roompath LIKE '商业、教学、教职工用电/%' AND is_active = TRUE;
```

Expected shape: active rooms should be close to live Upay count `4622`; `文昌校区` should be present; old commercial root should no longer appear in active path tree unless Upay starts returning it.

- [ ] **Step 6: Smoke path tree**

Call `/api/rooms/path-tree` with an authenticated token and verify root children include `文昌校区` and do not include stale-only roots.

- [ ] **Step 7: Rollback if needed**

If sync or deployment breaks production behavior, restore the DB backup and redeploy the previous release artifact.
