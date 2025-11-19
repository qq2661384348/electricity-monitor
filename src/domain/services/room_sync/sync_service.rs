//! 房间同步服务
//!
//! 负责从爬虫获取数据并同步到数据库

use std::sync::Arc;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::domain::models::{NewRoom, NewRoomPath};
use crate::domain::services::RoomSyncCache;
use crate::errors::{AppError, Result};
use crate::infrastructure::repositories::RoomRepository;
use crate::utils::hash::calculate_roompath_hash;

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

/// 房间同步服务
pub struct RoomSyncService {
    /// 房间仓储
    repository: Arc<RoomRepository>,
    
    /// 爬虫获取器
    fetcher: Arc<RoomFetcher>,
    
    /// 房间同步缓存
    cache: Arc<RoomSyncCache>,
    
    /// 默认电费阈值
    default_threshold: f32,
}

impl RoomSyncService {
    /// 创建新的同步服务实例
    pub fn new(
        repository: Arc<RoomRepository>,
        fetcher: Arc<RoomFetcher>,
        cache: Arc<RoomSyncCache>,
        default_threshold: f32,
    ) -> Self {
        Self {
            repository,
            fetcher,
            cache,
            default_threshold,
        }
    }
    
    /// 执行同步（使用缓存增量更新）
    /// 
    /// 从爬虫获取所有房间数据并同步到数据库
    /// 
    /// # 返回
    /// SyncStats - 同步统计信息
    /// 
    /// # 性能优化
    /// - 从缓存获取现有房间（零数据库查询）
    /// - 内存差异计算
    /// - 批量创建/更新数据库
    /// - 同步后增量更新缓存
    pub async fn sync(&self) -> Result<SyncStats> {
        tracing::info!("开始房间同步服务（缓存增量模式）...");
        
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
        
        // 5️⃣ 批量更新房间（逐个事务，因为涉及路径diff）
        for room_data in to_update {
            match self.update_room(&room_data).await {
                Ok(_) => {
                    stats.updated += 1;
                }
                Err(e) => {
                    tracing::error!(
                        "更新房间失败: roomid={}, error={}",
                        room_data.roomid,
                        e
                    );
                    stats.failed += 1;
                }
            }
        }
        
        // 6️⃣ 增量更新缓存（⭐ 保持同步）
        if !created_rooms.is_empty() {
            self.cache.add_rooms(created_rooms).await;
        }
        
        // 注意：更新的房间在update_room时已经通过兜底机制更新了缓存
        
        // 7️⃣ 完成统计
        stats.complete();
        stats.log();
        
        tracing::info!(
            "同步完成: 新增={}, 更新={}, 跳过={}, 失败={}",
            stats.new,
            stats.updated,
            stats.skipped,
            stats.failed
        );
        
        Ok(stats)
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
        // 这里返回true会触发update，在update中会做详细的路径diffing
        if new_has_additional {
            return true;
        }
        
        false
    }
    
    /// 批量创建房间
    async fn batch_create_rooms(&self, rooms_data: Vec<RoomData>) -> Result<Vec<crate::domain::models::Room>> {
        let new_rooms: Vec<NewRoom> = rooms_data
            .iter()
            .map(|room| {
                let primary_roompath_hash = calculate_roompath_hash(&room.primary_roompath);
                let has_additional_paths = room.path_count > 1;
                
                NewRoom {
                    roomid: room.roomid,
                    electricity_fee: 0.0,
                    threshold: self.default_threshold,
                    room_name: room.primary_roompath.split('/').next_back()
                        .unwrap_or("未知房间")
                        .to_string(),
                    primary_roompath: room.primary_roompath.clone(),
                    primary_roompath_hash,
                    has_additional_paths,
                    is_active: true,
                    source_type: "api_sync".to_string(),
                    external_id: None,
                    last_synced_at: Some(Utc::now().naive_utc()),
                }
            })
            .collect();
        
        // ⭐ 批量创建
        let created_rooms = self.repository.batch_create(new_rooms).await?;
        
        // 为每个新房间添加额外路径
        for room_data in &rooms_data {
            if room_data.path_count > 1 {
                let additional_paths: Vec<NewRoomPath> = room_data.roompaths
                    .iter()
                    .skip(1)
                    .map(|roompath| {
                        let roompath_hash = calculate_roompath_hash(roompath);
                        let room_name = roompath.split('/').next_back()
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
                    self.repository.add_additional_paths(additional_paths).await?;
                    
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
    
    /// 更新房间
    /// 
    /// 检查路径变化，更新主表和扩展表
    async fn update_room(&self, room: &RoomData) -> Result<()> {
        use std::collections::HashSet;
        
        // 1. 查询现有所有路径（聚合根）
        let existing_aggregate = self.repository.find_room_with_all_paths(room.roomid)
            .await?
            .ok_or_else(|| AppError::Internal(format!("房间不存在: roomid={}", room.roomid)))?;
        
        // 2. 构建路径集合用于对比
        let existing_paths: HashSet<String> = existing_aggregate.all_roompaths().into_iter().collect();
        let new_paths: HashSet<String> = room.roompaths.iter().cloned().collect();
        
        // 3. 检测变化
        let paths_to_add: Vec<&String> = new_paths.difference(&existing_paths).collect();
        let paths_to_remove: Vec<&String> = existing_paths.difference(&new_paths).collect();
        
        // 4. 如果主路径变化，需要更新主表
        if existing_aggregate.room.primary_roompath != room.primary_roompath {
            tracing::info!(
                "主路径变化: roomid={}, old={}, new={}",
                room.roomid,
                existing_aggregate.room.primary_roompath,
                room.primary_roompath
            );
            
            // 更新主表的primary_roompath和hash
            let new_hash = calculate_roompath_hash(&room.primary_roompath);
            self.repository.update_primary_roompath(
                room.roomid,
                &room.primary_roompath,
                new_hash
            ).await?;
        }
        
        // 5. 处理新增路径
        if !paths_to_add.is_empty() {
            let new_paths_to_insert: Vec<NewRoomPath> = paths_to_add
                .iter()
                .filter(|&&p| p != &room.primary_roompath)  // 排除主路径
                .map(|&roompath| {
                    let roompath_hash = calculate_roompath_hash(roompath);
                    let room_name = roompath.split('/').next_back()
                        .unwrap_or("未知房间")
                        .to_string();
                    
                    NewRoomPath {
                        roomid: room.roomid,
                        roompath: roompath.clone(),
                        roompath_hash,
                        room_name,
                        source_type: "api_sync".to_string(),
                    }
                })
                .collect();
            
            if !new_paths_to_insert.is_empty() {
                self.repository.add_additional_paths(new_paths_to_insert).await?;
                tracing::debug!("添加{}条新路径: roomid={}", paths_to_add.len(), room.roomid);
            }
        }
        
        // 6. 处理删除路径（物理删除）
        if !paths_to_remove.is_empty() {
            let remove_count = paths_to_remove.len();
            for path in paths_to_remove {
                if path != &existing_aggregate.room.primary_roompath {  // 不删除主路径
                    self.repository.delete_additional_path(room.roomid, path).await?;
                }
            }
            
            tracing::debug!("删除{}条旧路径: roomid={}", remove_count, room.roomid);
        }
        
        // 7. 更新has_additional_paths标志
        let final_additional_count = room.path_count.saturating_sub(1);  // 减去主路径
        let has_additional_paths = final_additional_count > 0;
        
        if has_additional_paths != existing_aggregate.room.has_additional_paths {
            self.repository.update_has_additional_paths(room.roomid, has_additional_paths).await?;
            
            tracing::debug!(
                "更新has_additional_paths: roomid={}, value={}",
                room.roomid,
                has_additional_paths
            );
        }
        
        tracing::info!("房间更新完成: roomid={}", room.roomid);
        Ok(())
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
