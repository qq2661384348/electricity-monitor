//! 房间同步服务（优化版）
//!
//! 负责从爬虫获取数据并同步到数据库
//! 主要优化：事务化更新、批量操作

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

use crate::domain::models::{NewRoom, NewRoomPath};
use crate::domain::services::RoomSyncCache;
use crate::errors::{AppError, Result};
use crate::infrastructure::repositories::RoomRepository;
use crate::infrastructure::DbPool;
use crate::utils::hash::calculate_roompath_hash;

use diesel_async::{AsyncConnection, AsyncPgConnection};

use super::crawler::{RoomData, RoomFetcher};

/// 同步统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    /// 新增房间数
    pub new: usize,

    /// 更新房间数
    pub updated: usize,

    /// 失败数
    pub failed: usize,

    /// 跳过数
    pub skipped: usize,

    /// 总处理数
    pub total: usize,

    /// 开始时间
    pub started_at: String,

    /// 完成时间
    pub completed_at: Option<String>,
}

impl SyncStats {
    fn new() -> Self {
        Self {
            new: 0,
            updated: 0,
            failed: 0,
            skipped: 0,
            total: 0,
            started_at: Utc::now().to_rfc3339(),
            completed_at: None,
        }
    }

    fn complete(&mut self) {
        self.completed_at = Some(Utc::now().to_rfc3339());
    }

    /// 输出统计日志
    pub fn log(&self) {
        tracing::info!(
            "同步完成: 总数={}, 新增={}, 更新={}, 失败={}, 跳过={}",
            self.total,
            self.new,
            self.updated,
            self.failed,
            self.skipped
        );
    }
}

/// 房间更新操作
#[derive(Debug, Clone)]
struct RoomUpdateOps {
    /// 房间ID
    roomid: i64,
    /// 新的主路径（如果变更）
    new_primary_path: Option<(String, i64)>, // (path, hash)
    /// 要添加的路径
    paths_to_add: Vec<NewRoomPath>,
    /// 要删除的路径
    paths_to_remove: Vec<String>,
    /// has_additional_paths标志的新值
    has_additional_paths: Option<bool>,
}

/// 房间同步服务（优化版）
pub struct RoomSyncService {
    /// 数据库连接池（用于事务）
    db_pool: Arc<DbPool>,

    /// 房间仓储
    repository: Arc<RoomRepository>,

    /// 爬虫获取器
    fetcher: Arc<RoomFetcher>,

    /// 房间同步缓存
    cache: Arc<RoomSyncCache>,

    /// 默认电费阈值
    default_threshold: f32,
}

/// 静态函数：执行房间更新操作（在事务中）
///
/// # 参数
/// - `conn`: 数据库连接（事务中）
/// - `ops`: 更新操作
async fn execute_room_update_ops_static(
    conn: &mut AsyncPgConnection,
    ops: &RoomUpdateOps,
) -> std::result::Result<(), diesel::result::Error> {
    use crate::infrastructure::database::schema::{room_paths, rooms};
    use diesel::prelude::*;

    // 1. 更新主路径（如果需要）
    if let Some((ref new_path, hash)) = ops.new_primary_path {
        diesel_async::RunQueryDsl::execute(
            diesel::update(rooms::table)
                .filter(rooms::roomid.eq(ops.roomid))
                .set((
                    rooms::primary_roompath.eq(new_path),
                    rooms::primary_roompath_hash.eq(hash),
                )),
            conn,
        )
        .await?;

        tracing::debug!("更新主路径: roomid={}", ops.roomid);
    }

    // 2. 添加新路径
    if !ops.paths_to_add.is_empty() {
        diesel_async::RunQueryDsl::execute(
            diesel::insert_into(room_paths::table)
                .values(&ops.paths_to_add)
                .on_conflict_do_nothing(),
            conn,
        )
        .await?;

        tracing::debug!(
            "添加 {} 条新路径: roomid={}",
            ops.paths_to_add.len(),
            ops.roomid
        );
    }

    // 3. 删除旧路径
    for path in &ops.paths_to_remove {
        diesel_async::RunQueryDsl::execute(
            diesel::delete(room_paths::table)
                .filter(room_paths::roomid.eq(ops.roomid))
                .filter(room_paths::roompath.eq(path)),
            conn,
        )
        .await?;
    }

    if !ops.paths_to_remove.is_empty() {
        tracing::debug!(
            "删除 {} 条旧路径: roomid={}",
            ops.paths_to_remove.len(),
            ops.roomid
        );
    }

    // 4. 更新has_additional_paths标志
    if let Some(has_additional) = ops.has_additional_paths {
        diesel_async::RunQueryDsl::execute(
            diesel::update(rooms::table)
                .filter(rooms::roomid.eq(ops.roomid))
                .set(rooms::has_additional_paths.eq(has_additional)),
            conn,
        )
        .await?;

        tracing::debug!(
            "更新has_additional_paths: roomid={}, value={}",
            ops.roomid,
            has_additional
        );
    }

    Ok(())
}

impl RoomSyncService {
    /// 创建新的同步服务实例
    pub fn new(
        db_pool: Arc<DbPool>,
        repository: Arc<RoomRepository>,
        fetcher: Arc<RoomFetcher>,
        cache: Arc<RoomSyncCache>,
        default_threshold: f32,
    ) -> Self {
        Self {
            db_pool,
            repository,
            fetcher,
            cache,
            default_threshold,
        }
    }

    /// 执行同步（使用缓存增量更新）
    ///
    /// # 返回
    /// SyncStats - 同步统计信息
    ///
    /// # 性能优化
    /// - 从缓存获取现有房间（零数据库查询）
    /// - 内存差异计算
    /// - 批量创建/更新数据库
    /// - 事务化更新（保证一致性）
    /// - 同步后增量更新缓存
    pub async fn sync(&self) -> Result<SyncStats> {
        tracing::info!("开始房间同步服务（缓存增量模式 + 事务化）...");

        let mut stats = SyncStats::new();

        // 1️⃣ 从爬虫获取最新数据
        let latest_rooms = self.fetcher.fetch_all().await?;

        stats.total = latest_rooms.len();
        tracing::info!("从爬虫获取到 {} 个房间数据", stats.total);

        // 2️⃣ 从缓存获取现有数据（⭐ 零数据库查询）
        let existing_map = self.cache.get_all_rooms().await;

        tracing::debug!("从缓存加载 {} 个现有房间", existing_map.len());

        // 3️⃣ 内存增量差异计算
        let mut to_create = Vec::new();
        let mut to_update = Vec::new();

        for room_data in latest_rooms {
            match existing_map.get(&room_data.roomid) {
                None => {
                    // 新增房间
                    to_create.push(room_data);
                }
                Some(existing_room) => {
                    // 检查是否需要更新（路径变化）
                    if self.needs_update(existing_room, &room_data) {
                        to_update.push(room_data);
                    } else {
                        stats.skipped += 1;
                    }
                }
            }
        }

        tracing::info!(
            "差异计算完成: 新增={}, 更新={}, 跳过={}",
            to_create.len(),
            to_update.len(),
            stats.skipped
        );

        // 4️⃣ 批量创建新房间（单次事务）
        let created_rooms = if !to_create.is_empty() {
            let create_count = to_create.len();
            match self.batch_create_rooms(to_create).await {
                Ok(rooms) => {
                    stats.new = rooms.len();
                    rooms
                }
                Err(e) => {
                    tracing::error!("批量创建失败: {}", e);
                    stats.failed += create_count;
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        // 5️⃣ 批量更新房间（优化：批量事务）
        if !to_update.is_empty() {
            let (updated_count, failed_count) =
                self.batch_update_rooms_transactional(to_update).await?;
            stats.updated = updated_count;
            stats.failed += failed_count;
        }

        // 6️⃣ 增量更新缓存（⭐ 保持同步）
        if !created_rooms.is_empty() {
            self.cache.add_rooms(created_rooms).await;
        }

        // 7️⃣ 完成统计
        stats.complete();
        stats.log();

        Ok(stats)
    }

    /// 批量更新房间（事务化）
    ///
    /// # 参数
    /// - `rooms_to_update`: 需要更新的房间数据
    ///
    /// # 返回
    /// (成功数, 失败数)
    ///
    /// # 优化策略
    /// - 批量预计算所有更新操作
    /// - 使用事务批量执行
    /// - 减少数据库往返次数
    async fn batch_update_rooms_transactional(
        &self,
        rooms_to_update: Vec<RoomData>,
    ) -> Result<(usize, usize)> {
        let mut success_count = 0;
        let mut failed_count = 0;

        // 批量预计算更新操作
        let mut update_ops_batch = Vec::new();

        for room_data in &rooms_to_update {
            match self.prepare_room_update_ops(room_data).await {
                Ok(ops) => update_ops_batch.push(ops),
                Err(e) => {
                    tracing::error!("准备更新操作失败: roomid={}, error={}", room_data.roomid, e);
                    failed_count += 1;
                }
            }
        }

        if update_ops_batch.is_empty() {
            return Ok((0, failed_count));
        }

        // 保存批量操作数量
        let batch_size = update_ops_batch.len();

        // 执行批量事务更新
        let mut conn = self.db_pool.get().await.map_err(|e| {
            AppError::Database(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(format!("获取数据库连接失败: {}", e)),
            ))
        })?;

        // 开始事务
        let transaction_result = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                let ops_to_process = update_ops_batch.clone();
                Box::pin(async move {
                    for ops in &ops_to_process {
                        // 执行单个房间的所有更新操作
                        execute_room_update_ops_static(conn, ops).await?;
                    }
                    Ok(batch_size)
                })
            })
            .await;

        match transaction_result {
            Ok(count) => {
                success_count = count;
                tracing::info!("批量事务更新成功: {} 个房间", count);
            }
            Err(e) => {
                tracing::error!("批量事务失败: {}", e);
                failed_count += batch_size;
            }
        }

        Ok((success_count, failed_count))
    }

    /// 准备房间更新操作
    ///
    /// # 参数
    /// - `room_data`: 新的房间数据
    ///
    /// # 返回
    /// 预计算的更新操作
    async fn prepare_room_update_ops(&self, room_data: &RoomData) -> Result<RoomUpdateOps> {
        // 1. 查询现有所有路径
        let existing_aggregate = self
            .repository
            .find_room_with_all_paths(room_data.roomid)
            .await?
            .ok_or_else(|| {
                AppError::Internal(format!("房间不存在: roomid={}", room_data.roomid))
            })?;

        // 2. 构建路径集合用于对比
        let existing_paths: HashSet<String> =
            existing_aggregate.all_roompaths().into_iter().collect();
        let new_paths: HashSet<String> = room_data.roompaths.iter().cloned().collect();

        // 3. 检测变化
        let paths_to_add: Vec<String> = new_paths
            .difference(&existing_paths)
            .filter(|&p| p != &room_data.primary_roompath)
            .cloned()
            .collect();

        let paths_to_remove: Vec<String> = existing_paths
            .difference(&new_paths)
            .filter(|&p| p != &existing_aggregate.room.primary_roompath)
            .cloned()
            .collect();

        // 4. 准备更新操作
        let mut ops = RoomUpdateOps {
            roomid: room_data.roomid,
            new_primary_path: None,
            paths_to_add: Vec::new(),
            paths_to_remove,
            has_additional_paths: None,
        };

        // 主路径变化
        if existing_aggregate.room.primary_roompath != room_data.primary_roompath {
            let new_hash = calculate_roompath_hash(&room_data.primary_roompath);
            ops.new_primary_path = Some((room_data.primary_roompath.clone(), new_hash));
        }

        // 新增路径
        ops.paths_to_add = paths_to_add
            .into_iter()
            .map(|roompath| {
                let roompath_hash = calculate_roompath_hash(&roompath);
                let room_name = roompath
                    .split('/')
                    .next_back()
                    .unwrap_or("未知房间")
                    .to_string();

                NewRoomPath {
                    roomid: room_data.roomid,
                    roompath,
                    roompath_hash,
                    room_name,
                    source_type: "api_sync".to_string(),
                }
            })
            .collect();

        // has_additional_paths标志
        let final_additional_count = room_data.path_count.saturating_sub(1);
        let has_additional_paths = final_additional_count > 0;

        if has_additional_paths != existing_aggregate.room.has_additional_paths {
            ops.has_additional_paths = Some(has_additional_paths);
        }

        Ok(ops)
    }

    /// 检查房间是否需要更新
    fn needs_update(&self, existing: &crate::domain::models::Room, new_data: &RoomData) -> bool {
        // 1. 主路径变化
        if existing.primary_roompath != new_data.primary_roompath {
            return true;
        }

        // 2. 路径数量变化
        let new_has_additional = new_data.path_count > 1;
        if existing.has_additional_paths != new_has_additional {
            return true;
        }

        // 3. 路径内容变化（简化判断，后续可优化）
        if new_has_additional {
            return true;
        }

        false
    }

    /// 批量创建房间（保持原有逻辑）
    async fn batch_create_rooms(
        &self,
        rooms_data: Vec<RoomData>,
    ) -> Result<Vec<crate::domain::models::Room>> {
        let new_rooms: Vec<NewRoom> = rooms_data
            .iter()
            .map(|room| {
                let primary_roompath_hash = calculate_roompath_hash(&room.primary_roompath);
                let has_additional_paths = room.path_count > 1;

                NewRoom {
                    roomid: room.roomid,
                    electricity_fee: 0.0,
                    threshold: self.default_threshold,
                    room_name: room
                        .primary_roompath
                        .split('/')
                        .next_back()
                        .unwrap_or("未知房间")
                        .to_string(),
                    primary_roompath: room.primary_roompath.clone(),
                    primary_roompath_hash,
                    has_additional_paths,
                    is_active: true,
                    source_type: "api_sync".to_string(),
                    external_id: None,
                    last_synced_at: Some(Utc::now().naive_utc()),
                    last_recovered_at: None,
                }
            })
            .collect();

        // ⭐ 批量创建
        let created_rooms = self.repository.batch_create(new_rooms).await?;

        // 为每个新房间添加额外路径
        for room_data in &rooms_data {
            if room_data.path_count > 1 {
                let additional_paths: Vec<NewRoomPath> = room_data
                    .roompaths
                    .iter()
                    .skip(1)
                    .map(|roompath| {
                        let roompath_hash = calculate_roompath_hash(roompath);
                        let room_name = roompath
                            .split('/')
                            .next_back()
                            .unwrap_or("未知房间")
                            .to_string();

                        NewRoomPath {
                            roomid: room_data.roomid,
                            roompath: roompath.clone(),
                            roompath_hash,
                            room_name,
                            source_type: "api_sync".to_string(),
                        }
                    })
                    .collect();

                if !additional_paths.is_empty() {
                    self.repository
                        .add_additional_paths(additional_paths)
                        .await?;

                    tracing::debug!(
                        "为房间 {} 添加了 {} 条额外路径",
                        room_data.roomid,
                        room_data.path_count - 1
                    );
                }
            }
        }

        Ok(created_rooms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_stats_new() {
        let stats = SyncStats::new();
        assert_eq!(stats.new, 0);
        assert_eq!(stats.updated, 0);
        assert_eq!(stats.failed, 0);
        assert_eq!(stats.total, 0);
        assert!(stats.completed_at.is_none());
    }

    #[test]
    fn test_sync_stats_complete() {
        let mut stats = SyncStats::new();
        stats.complete();
        assert!(stats.completed_at.is_some());
    }
}
