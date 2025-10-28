//! Room数据仓储实现
//! 
//! 提供Room实体的数据访问操作

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::domain::models::{NewRoom, ResetSendFlag, Room, UpdateElectricityFee, UpdateThreshold};
use crate::errors::{AppError, Result};
use crate::infrastructure::database::schema::rooms;
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

    /// 创建新房间
    /// 
    /// # 参数
    /// - `new_room`: 新房间数据
    /// 
    /// # 返回
    /// 创建成功的Room实体
    pub async fn create(&self, new_room: NewRoom) -> Result<Room> {
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::Internal(format!("Failed to get database connection: {}", e))
        })?;

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
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::Internal(format!("Failed to get database connection: {}", e))
        })?;

        rooms::table
            .find(id)
            .select(Room::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(AppError::Database)
    }

    /// 根据roomid查找房间（可能返回多个）
    /// 
    /// # 参数
    /// - `roomid`: 房间业务ID
    /// 
    /// # 返回
    /// 匹配的房间列表
    pub async fn find_by_roomid(&self, roomid: i32) -> Result<Vec<Room>> {
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::Internal(format!("Failed to get database connection: {}", e))
        })?;

        rooms::table
            .filter(rooms::roomid.eq(roomid))
            .select(Room::as_select())
            .load(&mut conn)
            .await
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
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::Internal(format!("Failed to get database connection: {}", e))
        })?;

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
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::Internal(format!("Failed to get database connection: {}", e))
        })?;

        let update = UpdateElectricityFee { electricity_fee };

        diesel::update(rooms::table.filter(rooms::roomid.eq(roomid)))
            .set(&update)
            .execute(&mut conn)
            .await
            .map_err(AppError::Database)
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
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::Internal(format!("Failed to get database connection: {}", e))
        })?;

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
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::Internal(format!("Failed to get database connection: {}", e))
        })?;

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
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::Internal(format!("Failed to get database connection: {}", e))
        })?;

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
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::Internal(format!("Failed to get database connection: {}", e))
        })?;

        rooms::table
            .select(Room::as_select())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
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
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::Internal(format!("Failed to get database connection: {}", e))
        })?;

        diesel::delete(rooms::table.find(id))
            .execute(&mut conn)
            .await
            .map_err(AppError::Database)
    }
}

#[cfg(test)]
mod tests {
    // 测试需要实际数据库连接，标记为ignore
    #[tokio::test]
    #[ignore]
    async fn test_create_room() {
        // 测试实现
    }
}
