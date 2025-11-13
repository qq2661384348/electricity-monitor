//! UserRoomBinding数据仓储实现
//! 
//! 提供用户-房间绑定关系的数据访问操作

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::domain::models::{NewUserRoomBinding, UpdateNotificationEnabled, UserRoomBinding};
use crate::errors::{AppError, Result};
use crate::infrastructure::database::schema::user_room_bindings;
use crate::infrastructure::DbPool;

/// UserRoomBinding数据仓储
#[derive(Clone)]
pub struct UserRoomBindingRepository {
    pool: DbPool,
}

impl UserRoomBindingRepository {
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

    /// 创建用户-房间绑定
    /// 
    /// # 参数
    /// - `binding`: 新绑定数据
    /// 
    /// # 返回
    /// 创建成功的绑定实体
    /// 
    /// # 错误
    /// - 用户不存在
    /// - 房间不存在
    /// - 绑定已存在（违反唯一约束）
    pub async fn create(&self, binding: NewUserRoomBinding) -> Result<UserRoomBinding> {
        let mut conn = self.get_conn().await?;

        let result = diesel::insert_into(user_room_bindings::table)
            .values(&binding)
            .returning(UserRoomBinding::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(AppError::Database)?;

        tracing::info!(
            user_id = %binding.user_id,
            roomid = binding.roomid,
            binding_id = %result.id,
            "创建用户-房间绑定成功"
        );

        Ok(result)
    }

    /// 根据ID查找绑定
    /// 
    /// # 参数
    /// - `id`: 绑定UUID
    /// 
    /// # 返回
    /// - `Some(UserRoomBinding)`: 找到绑定
    /// - `None`: 绑定不存在
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRoomBinding>> {
        let mut conn = self.get_conn().await?;

        user_room_bindings::table
            .find(id)
            .select(UserRoomBinding::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(AppError::Database)
    }

    /// 根据用户ID查找所有绑定
    /// 
    /// # 参数
    /// - `user_id`: 用户UUID
    /// 
    /// # 返回
    /// 用户的所有房间绑定列表
    pub async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<UserRoomBinding>> {
        let mut conn = self.get_conn().await?;

        user_room_bindings::table
            .filter(user_room_bindings::user_id.eq(user_id))
            .select(UserRoomBinding::as_select())
            .order_by(user_room_bindings::created_at.desc())
            .load(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 根据房间ID查找所有绑定
    /// 
    /// # 参数
    /// - `roomid`: 房间ID
    /// 
    /// # 返回
    /// 该房间的所有用户绑定列表
    pub async fn find_by_roomid(&self, roomid: i32) -> Result<Vec<UserRoomBinding>> {
        let mut conn = self.get_conn().await?;

        user_room_bindings::table
            .filter(user_room_bindings::roomid.eq(roomid))
            .select(UserRoomBinding::as_select())
            .order_by(user_room_bindings::created_at.desc())
            .load(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 查找启用通知的绑定（根据房间ID）
    /// 
    /// # 参数
    /// - `roomid`: 房间ID
    /// 
    /// # 返回
    /// 该房间启用通知的所有用户绑定列表
    /// 
    /// # 说明
    /// 用于通知服务查询需要发送通知的用户
    pub async fn find_active_bindings_by_roomid(&self, roomid: i32) -> Result<Vec<UserRoomBinding>> {
        let mut conn = self.get_conn().await?;

        user_room_bindings::table
            .filter(user_room_bindings::roomid.eq(roomid))
            .filter(user_room_bindings::notification_enabled.eq(true))
            .select(UserRoomBinding::as_select())
            .load(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 查找用户在特定房间的绑定
    /// 
    /// # 参数
    /// - `user_id`: 用户UUID
    /// - `roomid`: 房间ID
    /// 
    /// # 返回
    /// - `Some(UserRoomBinding)`: 找到绑定
    /// - `None`: 绑定不存在
    pub async fn find_by_user_and_room(&self, user_id: Uuid, roomid: i32) -> Result<Option<UserRoomBinding>> {
        let mut conn = self.get_conn().await?;

        user_room_bindings::table
            .filter(user_room_bindings::user_id.eq(user_id))
            .filter(user_room_bindings::roomid.eq(roomid))
            .select(UserRoomBinding::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(AppError::Database)
    }

    /// 更新通知开关
    /// 
    /// # 参数
    /// - `id`: 绑定UUID
    /// - `enabled`: 是否启用通知
    /// 
    /// # 返回
    /// 更新后的绑定实体
    pub async fn update_notification_enabled(&self, id: Uuid, enabled: bool) -> Result<UserRoomBinding> {
        let mut conn = self.get_conn().await?;

        let update = UpdateNotificationEnabled {
            notification_enabled: enabled,
        };

        let result = diesel::update(user_room_bindings::table.find(id))
            .set(&update)
            .returning(UserRoomBinding::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(AppError::Database)?;

        tracing::info!(
            binding_id = %id,
            notification_enabled = enabled,
            "更新通知开关成功"
        );

        Ok(result)
    }

    /// 删除绑定
    /// 
    /// # 参数
    /// - `id`: 绑定UUID
    /// 
    /// # 返回
    /// 删除的行数
    pub async fn delete(&self, id: Uuid) -> Result<usize> {
        let mut conn = self.get_conn().await?;

        let affected_rows = diesel::delete(user_room_bindings::table.find(id))
            .execute(&mut conn)
            .await
            .map_err(AppError::Database)?;

        tracing::info!(
            binding_id = %id,
            affected_rows = affected_rows,
            "删除绑定"
        );

        if affected_rows == 0 {
            tracing::warn!(
                binding_id = %id,
                "删除绑定但没有匹配的记录"
            );
        }

        Ok(affected_rows)
    }

    /// 检查绑定所有权
    /// 
    /// # 参数
    /// - `user_id`: 用户UUID
    /// - `binding_id`: 绑定UUID
    /// 
    /// # 返回
    /// - `true`: 用户拥有该绑定
    /// - `false`: 用户不拥有该绑定或绑定不存在
    /// 
    /// # 说明
    /// 用于权限验证，确保用户只能操作自己的绑定
    pub async fn check_ownership(&self, user_id: Uuid, binding_id: Uuid) -> Result<bool> {
        let mut conn = self.get_conn().await?;

        let count: i64 = user_room_bindings::table
            .filter(user_room_bindings::id.eq(binding_id))
            .filter(user_room_bindings::user_id.eq(user_id))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(AppError::Database)?;

        Ok(count > 0)
    }

    /// 批量查询多个房间的活跃绑定
    /// 
    /// # 参数
    /// - `roomids`: 房间ID列表
    /// 
    /// # 返回
    /// 所有房间的活跃绑定列表
    /// 
    /// # 说明
    /// 用于批量通知服务
    pub async fn find_active_bindings_by_roomids(&self, roomids: &[i32]) -> Result<Vec<UserRoomBinding>> {
        if roomids.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self.get_conn().await?;

        user_room_bindings::table
            .filter(user_room_bindings::roomid.eq_any(roomids))
            .filter(user_room_bindings::notification_enabled.eq(true))
            .select(UserRoomBinding::as_select())
            .load(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 删除用户的所有绑定
    /// 
    /// # 参数
    /// - `user_id`: 用户UUID
    /// 
    /// # 返回
    /// 删除的行数
    /// 
    /// # 说明
    /// 用于用户注销或删除
    pub async fn delete_all_by_user(&self, user_id: Uuid) -> Result<usize> {
        let mut conn = self.get_conn().await?;

        let affected_rows = diesel::delete(
            user_room_bindings::table.filter(user_room_bindings::user_id.eq(user_id))
        )
        .execute(&mut conn)
        .await
        .map_err(AppError::Database)?;

        tracing::info!(
            user_id = %user_id,
            affected_rows = affected_rows,
            "删除用户的所有绑定"
        );

        Ok(affected_rows)
    }

    /// 删除房间的所有绑定
    /// 
    /// # 参数
    /// - `roomid`: 房间ID
    /// 
    /// # 返回
    /// 删除的行数
    /// 
    /// # 说明
    /// 用于房间删除（通常由数据库级联删除处理）
    pub async fn delete_all_by_room(&self, roomid: i32) -> Result<usize> {
        let mut conn = self.get_conn().await?;

        let affected_rows = diesel::delete(
            user_room_bindings::table.filter(user_room_bindings::roomid.eq(roomid))
        )
        .execute(&mut conn)
        .await
        .map_err(AppError::Database)?;

        tracing::info!(
            roomid = roomid,
            affected_rows = affected_rows,
            "删除房间的所有绑定"
        );

        Ok(affected_rows)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_repository_creation() {
        // 测试仓储创建（不需要实际数据库连接）
        // 实际测试需要数据库环境
    }
}
