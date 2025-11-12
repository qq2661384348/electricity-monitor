//! Room数据仓储实现
//! 
//! 提供Room实体的数据访问操作

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::domain::models::{
    NewRoom, NewRoomPath, ResetSendFlag, Room, RoomAggregate, RoomPath,
    UpdateElectricityFee, UpdateThreshold,
};
use crate::errors::{AppError, Result};
use crate::infrastructure::database::schema::{room_paths, rooms};
use crate::infrastructure::DbPool;

/// Room数据仓储
#[derive(Clone)]
pub struct RoomRepository {
    pool: DbPool,
}

impl RoomRepository {
    /// 创建新的Repository实例
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 获取数据库连接（内部辅助方法）
    /// 
    /// # 返回
    /// 数据库连接或错误
    /// 
    /// # 错误
    /// 当连接池无法提供连接时返回`AppError::Internal`
    async fn get_conn(&self) -> Result<diesel_async::pooled_connection::deadpool::Object<diesel_async::AsyncPgConnection>> {
        self.pool.get().await.map_err(|e| {
            tracing::error!("Failed to get database connection: {}", e);
            AppError::Internal(format!("Failed to get database connection: {}", e))
        })
    }

    /// 创建新房间
    /// 
    /// # 参数
    /// - `new_room`: 新房间数据
    /// 
    /// # 返回
    /// 创建成功的Room实体
    pub async fn create(&self, new_room: NewRoom) -> Result<Room> {
        let mut conn = self.get_conn().await?;

        diesel::insert_into(rooms::table)
            .values(&new_room)
            .returning(Room::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 根据UUID查找房间
    /// 
    /// # 参数
    /// - `id`: 房间UUID
    /// 
    /// # 返回
    /// - `Some(Room)`: 找到房间
    /// - `None`: 房间不存在
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Room>> {
        let mut conn = self.get_conn().await?;

        rooms::table
            .find(id)
            .select(Room::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(AppError::Database)
    }

    /// 根据roomid查找房间（破坏性变更：roomid现为唯一约束）
    /// 
    /// # 参数
    /// - `roomid`: 房间业务ID
    /// 
    /// # 返回
    /// - `Some(Room)`: 找到房间
    /// - `None`: 房间不存在
    pub async fn find_by_roomid(&self, roomid: i32) -> Result<Option<Room>> {
        let mut conn = self.get_conn().await?;

        rooms::table
            .filter(rooms::roomid.eq(roomid))
            .select(Room::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(AppError::Database)
    }

    /// 更新房间电费
    /// 
    /// # 参数
    /// - `id`: 房间UUID
    /// - `update`: 电费更新数据
    /// 
    /// # 返回
    /// 更新后的Room实体
    /// 
    /// # 注意
    /// 触发器会自动检查电费是否超过阈值并更新send_flag
    pub async fn update_electricity_fee(
        &self,
        id: Uuid,
        update: UpdateElectricityFee,
    ) -> Result<Room> {
        let mut conn = self.get_conn().await?;

        diesel::update(rooms::table.find(id))
            .set(&update)
            .returning(Room::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 批量更新电费（使用roomid）
    /// 
    /// # 参数
    /// - `roomid`: 房间业务ID
    /// - `electricity_fee`: 新的电费值
    /// 
    /// # 返回
    /// 更新的房间数量
    /// 
    /// # 说明
    /// 用于电费插入服务，UPDATE覆盖旧值
    pub async fn update_electricity_fee_by_roomid(
        &self,
        roomid: i32,
        electricity_fee: f32,
    ) -> Result<usize> {
        let mut conn = self.get_conn().await?;

        let update = UpdateElectricityFee { electricity_fee };

        let affected_rows = diesel::update(rooms::table.filter(rooms::roomid.eq(roomid)))
            .set(&update)
            .execute(&mut conn)
            .await
            .map_err(AppError::Database)?;
        
        // 记录批量更新操作
        tracing::info!(
            roomid = roomid,
            electricity_fee = electricity_fee,
            affected_rows = affected_rows,
            "批量更新电费完成"
        );
        
        // 异常情况警告
        if affected_rows == 0 {
            tracing::warn!(
                roomid = roomid,
                "批量更新电费但没有匹配的房间"
            );
        }
        
        Ok(affected_rows)
    }

    /// 更新房间阈值
    /// 
    /// # 参数
    /// - `id`: 房间UUID
    /// - `update`: 阈值更新数据
    /// 
    /// # 返回
    /// 更新后的Room实体
    pub async fn update_threshold(&self, id: Uuid, update: UpdateThreshold) -> Result<Room> {
        let mut conn = self.get_conn().await?;

        diesel::update(rooms::table.find(id))
            .set(&update)
            .returning(Room::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 重置房间的send_flag为false
    /// 
    /// # 参数
    /// - `id`: 房间UUID
    /// 
    /// # 返回
    /// 更新后的Room实体
    pub async fn reset_send_flag(&self, id: Uuid) -> Result<Room> {
        let mut conn = self.get_conn().await?;

        let reset = ResetSendFlag::new();

        diesel::update(rooms::table.find(id))
            .set(&reset)
            .returning(Room::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 查询所有send_flag为true的房间
    /// 
    /// # 返回
    /// 需要发送通知的房间列表
    pub async fn find_rooms_with_send_flag_true(&self) -> Result<Vec<Room>> {
        let mut conn = self.get_conn().await?;

        rooms::table
            .filter(rooms::send_flag.eq(true))
            .select(Room::as_select())
            .load(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 查询所有房间（分页）
    /// 
    /// # 参数
    /// - `limit`: 每页数量
    /// - `offset`: 偏移量
    /// 
    /// # 返回
    /// 房间列表
    pub async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<Room>> {
        let mut conn = self.get_conn().await?;

        rooms::table
            .select(Room::as_select())
            .order_by(rooms::created_at.desc())  // 按创建时间降序排序
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 根据roompath查找房间
    /// 
    /// # 参数
    /// - `roompath`: 房间路径
    /// 
    /// # 返回
    /// - `Some(Room)`: 找到房间
    /// - `None`: 房间不存在
    pub async fn find_by_roompath(&self, roompath: &str) -> Result<Option<Room>> {
        let mut conn = self.get_conn().await?;

        rooms::table
            .filter(rooms::primary_roompath.eq(roompath))
            .select(Room::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(AppError::Database)
    }

    /// 根据primary_roompath_hash查找房间（第一级查询）
    /// 
    /// # 参数
    /// - `hash`: roompath哈希值
    /// - `roompath`: 实际路径（用于精确验证，防止哈希冲突）
    /// 
    /// # 返回
    /// - `Some(Room)`: 找到房间
    /// - `None`: 房间不存在
    pub async fn find_by_primary_roompath_hash(&self, hash: i64, roompath: &str) -> Result<Option<Room>> {
        let mut conn = self.get_conn().await?;

        rooms::table
            .filter(rooms::primary_roompath_hash.eq(hash))
            .filter(rooms::primary_roompath.eq(roompath))  // ⭐ 精确验证（防哈希冲突）
            .select(Room::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(AppError::Database)
    }
    
    /// 根据额外路径查询roomid（第二级查询）
    /// 
    /// # 参数
    /// - `hash`: roompath哈希值
    /// - `roompath`: 实际路径（用于精确验证）
    /// 
    /// # 返回
    /// - `Some(i32)`: 找到的roomid
    /// - `None`: 路径不存在
    pub async fn find_roomid_by_additional_roompath(&self, hash: i64, roompath: &str) -> Result<Option<i32>> {
        use crate::infrastructure::database::schema::room_paths;
        
        let mut conn = self.get_conn().await?;

        room_paths::table
            .filter(room_paths::roompath_hash.eq(hash))
            .filter(room_paths::roompath.eq(roompath))  // ⭐ 精确验证
            .select(room_paths::roomid)
            .first(&mut conn)
            .await
            .optional()
            .map_err(AppError::Database)
    }

    /// 查找房间及其所有路径（聚合根）
    /// 
    /// # 参数
    /// - `roomid`: 房间业务ID
    /// 
    /// # 返回
    /// - `Some(RoomAggregate)`: 找到房间及其所有路径
    /// - `None`: 房间不存在
    pub async fn find_room_with_all_paths(&self, roomid: i32) -> Result<Option<RoomAggregate>> {
        let mut conn = self.get_conn().await?;

        // 查找房间
        let room = match self.find_by_roomid(roomid).await? {
            Some(r) => r,
            None => return Ok(None),
        };

        // 查找额外路径
        let additional_paths = room_paths::table
            .filter(room_paths::roomid.eq(roomid))
            .select(RoomPath::as_select())
            .load(&mut conn)
            .await
            .map_err(AppError::Database)?;

        Ok(Some(RoomAggregate::new(room, additional_paths)))
    }

    /// 查找房间的额外路径
    /// 
    /// # 参数
    /// - `roomid`: 房间业务ID
    /// 
    /// # 返回
    /// 额外路径列表
    pub async fn find_additional_paths(&self, roomid: i32) -> Result<Vec<RoomPath>> {
        let mut conn = self.get_conn().await?;

        room_paths::table
            .filter(room_paths::roomid.eq(roomid))
            .select(RoomPath::as_select())
            .load(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 更新房间的主路径信息
    /// 
    /// # 参数
    /// - `roomid`: 房间业务ID
    /// - `new_primary_roompath`: 新的主路径
    /// - `new_hash`: 新的哈希值
    /// 
    /// # 返回
    /// 更新的行数
    pub async fn update_primary_roompath(&self, roomid: i32, new_primary_roompath: &str, new_hash: i64) -> Result<usize> {
        let mut conn = self.get_conn().await?;
        
        diesel::update(rooms::table.filter(rooms::roomid.eq(roomid)))
            .set((
                rooms::primary_roompath.eq(new_primary_roompath),
                rooms::primary_roompath_hash.eq(new_hash),
                rooms::last_synced_at.eq(Some(chrono::Utc::now().naive_utc())),
            ))
            .execute(&mut conn)
            .await
            .map_err(AppError::Database)
    }
    
    /// 删除房间的额外路径
    /// 
    /// # 参数
    /// - `roomid`: 房间业务ID
    /// - `roompath`: 要删除的路径
    /// 
    /// # 返回
    /// 删除的行数
    pub async fn delete_additional_path(&self, roomid: i32, roompath: &str) -> Result<usize> {
        use crate::infrastructure::database::schema::room_paths;
        
        let mut conn = self.get_conn().await?;
        
        diesel::delete(
            room_paths::table
                .filter(room_paths::roomid.eq(roomid))
                .filter(room_paths::roompath.eq(roompath))
        )
        .execute(&mut conn)
        .await
        .map_err(AppError::Database)
    }
    
    /// 更新房间的has_additional_paths标志
    /// 
    /// # 参数
    /// - `roomid`: 房间业务ID
    /// - `has_additional`: 是否有额外路径
    /// 
    /// # 返回
    /// 更新的行数
    pub async fn update_has_additional_paths(&self, roomid: i32, has_additional: bool) -> Result<usize> {
        let mut conn = self.get_conn().await?;
        
        diesel::update(rooms::table.filter(rooms::roomid.eq(roomid)))
            .set(rooms::has_additional_paths.eq(has_additional))
            .execute(&mut conn)
            .await
            .map_err(AppError::Database)
    }
    
    /// 添加额外的房间路径
    /// 
    /// # 参数
    /// - `new_paths`: 新路径列表
    /// 
    /// # 返回
    /// 创建的RoomPath列表
    pub async fn add_additional_paths(&self, new_paths: Vec<NewRoomPath>) -> Result<Vec<RoomPath>> {
        let mut conn = self.get_conn().await?;

        diesel::insert_into(room_paths::table)
            .values(&new_paths)
            .returning(RoomPath::as_returning())
            .get_results(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 停用除指定roomid列表外的所有房间
    /// 
    /// # 参数
    /// - `active_roomids`: 要保持激活的roomid列表
    /// 
    /// # 返回
    /// 停用的房间数量
    pub async fn deactivate_except(&self, active_roomids: &[i32]) -> Result<usize> {
        let mut conn = self.get_conn().await?;

        diesel::update(
            rooms::table
                .filter(rooms::roomid.ne_all(active_roomids))
                .filter(rooms::is_active.eq(true))
        )
        .set(rooms::is_active.eq(false))
        .execute(&mut conn)
        .await
        .map_err(AppError::Database)
    }

    /// 删除房间
    /// 
    /// # 参数
    /// - `id`: 房间UUID
    /// 
    /// # 返回
    /// 删除的房间数量
    pub async fn delete(&self, id: Uuid) -> Result<usize> {
        let mut conn = self.get_conn().await?;

        diesel::delete(rooms::table.find(id))
            .execute(&mut conn)
            .await
            .map_err(AppError::Database)
    }
    
    // === 同步日志相关方法 ===
    
    /// 查询同步历史记录
    /// 
    /// # 参数
    /// - `limit`: 最多返回的记录数
    /// 
    /// # 返回
    /// 同步日志列表（按时间倒序）
    pub async fn get_sync_history(&self, limit: i64) -> Result<Vec<crate::domain::models::RoomSyncLog>> {
        use crate::infrastructure::database::schema::room_sync_log;
        
        let mut conn = self.get_conn().await?;
        
        room_sync_log::table
            .order(room_sync_log::started_at.desc())
            .limit(limit)
            .select(crate::domain::models::RoomSyncLog::as_select())
            .load(&mut conn)
            .await
            .map_err(AppError::Database)
    }
    
    /// 创建同步日志记录
    pub async fn create_sync_log(&self, log: crate::domain::models::NewRoomSyncLog) -> Result<crate::domain::models::RoomSyncLog> {
        use crate::infrastructure::database::schema::room_sync_log;
        
        let mut conn = self.get_conn().await?;
        
        diesel::insert_into(room_sync_log::table)
            .values(&log)
            .returning(crate::domain::models::RoomSyncLog::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(AppError::Database)
    }
    
    /// 更新同步日志记录
    pub async fn update_sync_log(&self, id: Uuid, update: crate::domain::models::UpdateRoomSyncLog) -> Result<crate::domain::models::RoomSyncLog> {
        use crate::infrastructure::database::schema::room_sync_log;
        
        let mut conn = self.get_conn().await?;
        
        diesel::update(room_sync_log::table.find(id))
            .set(&update)
            .returning(crate::domain::models::RoomSyncLog::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(AppError::Database)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::database::DatabaseType;
    use crate::infrastructure::database::create_pool;
    use crate::domain::models::NewRoom;
    use std::sync::Arc;

    async fn setup_test_pool() -> DbPool {
        let config = crate::config::DatabaseConfig {
            db_type: DatabaseType::Postgres,
            host: "47.92.117.121".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            database: "electricity_dev".to_string(),
            max_connections: 5,
            min_connections: 1,
            connection_timeout: 30,
        };
        create_pool(&config).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_room() {
        let pool = setup_test_pool().await;
        let repo = RoomRepository::new(pool.clone());

        let new_room = NewRoom {
            roomid: 99999,  // 使用特殊ID避免冲突
            electricity_fee: 0.0,
            threshold: 100.0,
            room_name: "测试房间".to_string(),
            primary_roompath: "测试/路径/99999".to_string(),
            primary_roompath_hash: crate::utils::hash::calculate_roompath_hash("测试/路径/99999"),
            has_additional_paths: false,
            is_active: true,
            source_type: "test".to_string(),
            external_id: None,
            last_synced_at: None,
        };

        let result = repo.create(new_room).await;
        assert!(result.is_ok(), "创建房间失败: {:?}", result.err());

        // 清理测试数据
        if let Ok(room) = result {
            let _ = repo.delete(room.id).await;
        }
    }

    #[tokio::test]
    async fn test_find_by_roomid() {
        let pool = setup_test_pool().await;
        let repo = RoomRepository::new(pool);

        // 查询一个可能存在的roomid
        let result = repo.find_by_roomid(1).await;
        assert!(result.is_ok(), "查询失败: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_get_sync_history() {
        let pool = setup_test_pool().await;
        let repo = RoomRepository::new(pool);

        let result = repo.get_sync_history(10).await;
        assert!(result.is_ok(), "查询同步历史失败: {:?}", result.err());
    }
}
