//! 房间同步缓存
//!
//! 缓存所有活跃房间及其路径，支持增量更新，避免频繁查询数据库

use crate::domain::models::{Room, RoomPath};
use crate::errors::Result;
use crate::infrastructure::{repositories::RoomRepository, DbPool};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 房间同步缓存
///
/// 使用Arc<RwLock<HashMap>>实现线程安全的读写锁
pub struct RoomSyncCache {
    /// 主缓存：roomid → Room
    rooms: Arc<RwLock<HashMap<i32, Room>>>,
    
    /// 路径缓存：roomid → Vec<RoomPath>
    paths: Arc<RwLock<HashMap<i32, Vec<RoomPath>>>>,
    
    /// 数据库连接池（用于刷新缓存）
    pool: DbPool,
}

impl RoomSyncCache {
    /// 创建房间同步缓存（初始化时自动加载）
    ///
    /// # 参数
    /// - `pool`: 数据库连接池
    ///
    /// # 返回
    /// RoomSyncCache实例
    ///
    /// # 错误
    /// 如果初始加载失败，返回数据库错误
    pub async fn new(pool: DbPool) -> Result<Self> {
        let rooms = Arc::new(RwLock::new(HashMap::new()));
        let paths = Arc::new(RwLock::new(HashMap::new()));
        
        let cache = Self {
            rooms,
            paths,
            pool,
        };
        
        // ⭐ 启动时唯一的全量数据库查询
        cache.full_refresh().await?;
        
        Ok(cache)
    }

    /// 全量刷新缓存（仅启动时 + 手动触发）
    ///
    /// # 说明
    /// - 查询所有is_active=true的房间
    /// - 查询所有额外路径
    /// - 构建HashMap缓存
    ///
    /// # 错误
    /// 如果查询失败，返回数据库错误
    pub async fn full_refresh(&self) -> Result<()> {
        let repo = RoomRepository::new(self.pool.clone());
        
        tracing::info!("开始全量刷新RoomSyncCache");
        
        // 1. 查询所有活跃房间
        let all_rooms = repo.find_all_active().await?;
        
        tracing::debug!("查询到 {} 个活跃房间", all_rooms.len());
        
        // 2. 查询所有额外路径
        let all_paths = repo.find_all_additional_paths().await?;
        
        tracing::debug!("查询到 {} 条额外路径", all_paths.len());
        
        // 3. 构建房间缓存
        let rooms_map: HashMap<i32, Room> = all_rooms
            .into_iter()
            .map(|r| (r.roomid, r))
            .collect();
        
        // 4. 构建路径缓存（按roomid分组）
        let mut paths_map: HashMap<i32, Vec<RoomPath>> = HashMap::new();
        for path in all_paths {
            paths_map
                .entry(path.roomid)
                .or_default()
                .push(path);
        }
        
        // 5. 更新缓存
        *self.rooms.write().await = rooms_map;
        *self.paths.write().await = paths_map;
        
        // 6. 记录统计（先获取值再记录日志，避免Send问题）
        let room_count = self.rooms.read().await.len();
        let path_groups = self.paths.read().await.len();
        
        tracing::info!(
            room_count = room_count,
            path_groups = path_groups,
            "RoomSyncCache 全量刷新完成"
        );
        
        Ok(())
    }

    /// 增量添加房间（不查询数据库）
    ///
    /// # 参数
    /// - `new_rooms`: 新创建的房间列表
    ///
    /// # 说明
    /// - 将新房间添加到缓存
    /// - 纯内存操作，微秒级延迟
    pub async fn add_rooms(&self, new_rooms: Vec<Room>) {
        if new_rooms.is_empty() {
            return;
        }
        
        let mut cache = self.rooms.write().await;
        for room in new_rooms {
            cache.insert(room.roomid, room);
        }
        
        tracing::debug!(
            count = cache.len(),
            "增量添加房间到缓存"
        );
    }

    /// 增量更新房间（不查询数据库）
    ///
    /// # 参数
    /// - `updated_rooms`: 更新的房间列表
    ///
    /// # 说明
    /// - 更新缓存中的现有房间
    /// - 纯内存操作，微秒级延迟
    pub async fn update_rooms(&self, updated_rooms: Vec<Room>) {
        if updated_rooms.is_empty() {
            return;
        }
        
        let mut cache = self.rooms.write().await;
        for room in updated_rooms {
            cache.insert(room.roomid, room);
        }
        
        tracing::debug!(
            count = cache.len(),
            "增量更新房间缓存"
        );
    }

    /// 增量更新路径缓存
    ///
    /// # 参数
    /// - `roomid`: 房间ID
    /// - `paths`: 该房间的所有路径
    pub async fn update_paths(&self, roomid: i32, paths: Vec<RoomPath>) {
        let mut cache = self.paths.write().await;
        
        if paths.is_empty() {
            cache.remove(&roomid);
        } else {
            cache.insert(roomid, paths);
        }
    }

    /// 获取所有房间（零数据库查询）
    ///
    /// # 返回
    /// roomid → Room的HashMap克隆
    ///
    /// # 说明
    /// - 使用读锁，支持并发读取
    /// - 返回克隆，避免锁持有时间过长
    pub async fn get_all_rooms(&self) -> HashMap<i32, Room> {
        self.rooms.read().await.clone()
    }

    /// 获取单个房间（带兜底机制）
    ///
    /// # 参数
    /// - `roomid`: 房间ID
    ///
    /// # 返回
    /// - `Some(Room)`: 找到房间
    /// - `None`: 房间不存在
    ///
    /// # 说明
    /// - 优先从缓存读取
    /// - 缓存未命中时降级到数据库查询（兜底）
    pub async fn get_room(&self, roomid: i32) -> Result<Option<Room>> {
        // 1. 尝试从缓存获取
        {
            let cache = self.rooms.read().await;
            if let Some(room) = cache.get(&roomid) {
                return Ok(Some(room.clone()));
            }
        }
        
        // 2. 缓存未命中，降级到数据库（兜底）
        tracing::warn!(
            roomid = roomid,
            "缓存未命中，降级到数据库查询"
        );
        
        let repo = RoomRepository::new(self.pool.clone());
        let room = repo.find_by_roomid(roomid).await?;
        
        // 3. 更新缓存
        if let Some(ref r) = room {
            let mut cache = self.rooms.write().await;
            cache.insert(roomid, r.clone());
        }
        
        Ok(room)
    }

    /// 获取房间的所有路径
    ///
    /// # 参数
    /// - `roomid`: 房间ID
    ///
    /// # 返回
    /// 路径列表（如果不存在返回空Vec）
    pub async fn get_paths(&self, roomid: i32) -> Vec<RoomPath> {
        self.paths
            .read()
            .await
            .get(&roomid)
            .cloned()
            .unwrap_or_default()
    }

    /// 移除房间（软删除/停用）
    ///
    /// # 参数
    /// - `roomid`: 房间ID
    pub async fn remove_room(&self, roomid: i32) {
        self.rooms.write().await.remove(&roomid);
        self.paths.write().await.remove(&roomid);
        
        tracing::debug!(
            roomid = roomid,
            "从缓存中移除房间"
        );
    }

    /// 获取缓存大小
    ///
    /// # 返回
    /// (房间数量, 路径组数量)
    pub async fn size(&self) -> (usize, usize) {
        let room_count = self.rooms.read().await.len();
        let path_count = self.paths.read().await.len();
        (room_count, path_count)
    }

    /// 检查缓存是否为空
    ///
    /// # 返回
    /// true表示缓存为空
    pub async fn is_empty(&self) -> bool {
        self.rooms.read().await.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_sync_cache_struct() {
        // 测试结构体可以正常构造（编译时验证）
        // Arc<RwLock<HashMap>>: 8 bytes * 2 + DbPool (Arc): 8 bytes = 24 bytes
        assert_eq!(std::mem::size_of::<RoomSyncCache>(), 24);
    }
}
