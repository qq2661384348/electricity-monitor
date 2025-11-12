//! 房间同步服务
//!
//! 负责从爬虫获取数据并同步到数据库

use std::sync::Arc;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::domain::models::{NewRoom, NewRoomPath};
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
    
    /// 默认电费阈值
    default_threshold: f32,
}

impl RoomSyncService {
    /// 创建新的同步服务实例
    pub fn new(
        repository: Arc<RoomRepository>,
        fetcher: Arc<RoomFetcher>,
        default_threshold: f32,
    ) -> Self {
        Self {
            repository,
            fetcher,
            default_threshold,
        }
    }
    
    /// 执行同步
    /// 
    /// 从爬虫获取所有房间数据并同步到数据库
    /// 
    /// # 返回
    /// SyncStats - 同步统计信息
    pub async fn sync(&self) -> Result<SyncStats> {
        tracing::info!("开始房间同步服务...");
        
        let mut stats = SyncStats::new();
        
        // 1. 调用爬虫获取数据
        let rooms = self.fetcher.fetch_all().await?;
        
        stats.total = rooms.len();
        tracing::info!("获取到{}个房间数据", stats.total);
        
        // 2. 遍历同步
        for room in rooms {
            match self.sync_room(&room).await {
                Ok(is_new) => {
                    if is_new {
                        stats.new += 1;
                    } else {
                        stats.updated += 1;
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "同步房间失败: roomid={}, error={}",
                        room.roomid,
                        e
                    );
                    stats.failed += 1;
                }
            }
        }
        
        // 3. 完成统计
        stats.complete();
        stats.log();
        
        Ok(stats)
    }
    
    /// 同步单个房间
    /// 
    /// # 参数
    /// - `room`: 爬虫获取的房间数据
    /// 
    /// # 返回
    /// bool - true表示新增，false表示更新
    async fn sync_room(&self, room: &RoomData) -> Result<bool> {
        // 检查房间是否已存在
        let existing = self.repository.find_by_roomid(room.roomid).await?;
        
        if existing.is_none() {
            // 新增房间
            self.create_room(room).await?;
            tracing::info!("新增房间: roomid={}", room.roomid);
            Ok(true)
        } else {
            // 更新房间
            self.update_room(room).await?;
            tracing::debug!("更新房间: roomid={}", room.roomid);
            Ok(false)
        }
    }
    
    /// 创建新房间
    /// 
    /// ⭐ 应用层维护 has_additional_paths（非触发器）
    async fn create_room(&self, room: &RoomData) -> Result<()> {
        // 计算主路径哈希
        let primary_roompath_hash = calculate_roompath_hash(&room.primary_roompath);
        
        // ⭐ 应用层计算 has_additional_paths
        let has_additional_paths = room.path_count > 1;
        
        // 创建主表记录
        let new_room = NewRoom {
            roomid: room.roomid,
            electricity_fee: 0.0,  // 初始电费为0
            threshold: self.default_threshold,
            room_name: room.primary_roompath.split('/').next_back()
                .unwrap_or("未知房间")
                .to_string(),
            primary_roompath: room.primary_roompath.clone(),
            primary_roompath_hash,
            has_additional_paths,  // ⭐ 应用层维护
            is_active: true,
            source_type: "api_sync".to_string(),
            external_id: None,
            last_synced_at: Some(Utc::now().naive_utc()),
        };
        
        self.repository.create(new_room).await?;
        
        // 如果有额外路径，插入扩展表
        if has_additional_paths {
            let additional_paths: Vec<NewRoomPath> = room.roompaths
                .iter()
                .skip(1)  // 跳过主路径
                .map(|roompath| {
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
            
            if !additional_paths.is_empty() {
                self.repository.add_additional_paths(additional_paths).await?;
                tracing::debug!(
                    "添加{}条额外路径: roomid={}",
                    room.path_count - 1,
                    room.roomid
                );
            }
        }
        
        Ok(())
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
