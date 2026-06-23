//! UserRoomBinding数据仓储实现
//!
//! 提供用户-房间绑定关系的数据访问操作

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::domain::models::{
    NewUserRoomBinding, UpdateLastNotified, UpdateNotificationEnabled, UserRoomBinding,
};
use crate::errors::{AppError, Result};
use crate::infrastructure::database::schema::user_room_bindings;
use crate::infrastructure::DbPool;
use chrono::NaiveDateTime;

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
    async fn get_conn(
        &self,
    ) -> Result<diesel_async::pooled_connection::deadpool::Object<diesel_async::AsyncPgConnection>>
    {
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
    pub async fn find_by_roomid(&self, roomid: i64) -> Result<Vec<UserRoomBinding>> {
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
    pub async fn find_active_bindings_by_roomid(
        &self,
        roomid: i64,
    ) -> Result<Vec<UserRoomBinding>> {
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
    pub async fn find_by_user_and_room(
        &self,
        user_id: Uuid,
        roomid: i64,
    ) -> Result<Option<UserRoomBinding>> {
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
    pub async fn update_notification_enabled(
        &self,
        id: Uuid,
        enabled: bool,
    ) -> Result<UserRoomBinding> {
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
    pub async fn find_active_bindings_by_roomids(
        &self,
        roomids: &[i64],
    ) -> Result<Vec<UserRoomBinding>> {
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
            user_room_bindings::table.filter(user_room_bindings::user_id.eq(user_id)),
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
    pub async fn delete_all_by_room(&self, roomid: i64) -> Result<usize> {
        let mut conn = self.get_conn().await?;

        let affected_rows =
            diesel::delete(user_room_bindings::table.filter(user_room_bindings::roomid.eq(roomid)))
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

    // ==================== 通知状态持久化方法 ====================

    /// 更新用户-房间绑定的最后通知时间
    ///
    /// # 参数
    /// - `user_id`: 用户UUID
    /// - `roomid`: 房间ID
    /// - `time`: 通知时间
    ///
    /// # 返回
    /// 更新的行数
    ///
    /// # 说明
    /// 用于在发送通知后持久化通知状态，防止服务器重启后重复通知
    pub async fn update_last_notified(
        &self,
        user_id: Uuid,
        roomid: i64,
        time: NaiveDateTime,
    ) -> Result<usize> {
        let mut conn = self.get_conn().await?;

        let update = UpdateLastNotified {
            last_notified_at: Some(time),
        };

        let affected_rows = diesel::update(
            user_room_bindings::table
                .filter(user_room_bindings::user_id.eq(user_id))
                .filter(user_room_bindings::roomid.eq(roomid)),
        )
        .set(&update)
        .execute(&mut conn)
        .await
        .map_err(AppError::Database)?;

        if affected_rows > 0 {
            tracing::debug!(
                user_id = %user_id,
                roomid = roomid,
                time = %time,
                "更新最后通知时间成功"
            );
        }

        Ok(affected_rows)
    }

    /// 重置用户-房间绑定的最后通知时间（设为NULL）
    ///
    /// # 参数
    /// - `user_id`: 用户UUID
    /// - `roomid`: 房间ID
    ///
    /// # 返回
    /// 更新的行数
    ///
    /// # 说明
    /// 用于在房间电费恢复且过了观察期后重置通知状态
    pub async fn reset_last_notified(&self, user_id: Uuid, roomid: i64) -> Result<usize> {
        let mut conn = self.get_conn().await?;

        // 直接使用 DSL 设置 NULL，避免 AsChangeset 结构体的 None 跳过行为
        // 参考: https://github.com/diesel-rs/diesel/issues/885
        let affected_rows = diesel::update(
            user_room_bindings::table
                .filter(user_room_bindings::user_id.eq(user_id))
                .filter(user_room_bindings::roomid.eq(roomid)),
        )
        .set(user_room_bindings::last_notified_at.eq(None::<NaiveDateTime>))
        .execute(&mut conn)
        .await
        .map_err(AppError::Database)?;

        if affected_rows > 0 {
            tracing::debug!(
                user_id = %user_id,
                roomid = roomid,
                "重置最后通知时间成功"
            );
        }

        Ok(affected_rows)
    }

    /// 批量重置房间的所有绑定的最后通知时间
    ///
    /// # 参数
    /// - `roomid`: 房间ID
    ///
    /// # 返回
    /// 更新的行数
    ///
    /// # 说明
    /// 用于在房间电费恢复且过了观察期后批量重置该房间所有用户的通知状态
    pub async fn reset_last_notified_by_roomid(&self, roomid: i64) -> Result<usize> {
        let mut conn = self.get_conn().await?;

        // 直接使用 DSL 设置 NULL，避免 AsChangeset 结构体的 None 跳过行为
        // 参考: https://github.com/diesel-rs/diesel/issues/885
        let affected_rows =
            diesel::update(user_room_bindings::table.filter(user_room_bindings::roomid.eq(roomid)))
                .set(user_room_bindings::last_notified_at.eq(None::<NaiveDateTime>))
                .execute(&mut conn)
                .await
                .map_err(AppError::Database)?;

        if affected_rows > 0 {
            tracing::info!(
                roomid = roomid,
                affected_rows = affected_rows,
                "批量重置房间的最后通知时间"
            );
        }

        Ok(affected_rows)
    }

    /// 加载所有有通知历史的绑定记录
    ///
    /// # 返回
    /// 包含 `(user_id, roomid, last_notified_at)` 的元组列表
    ///
    /// # 说明
    /// 用于服务器启动时从数据库恢复通知历史状态到内存
    pub async fn find_all_with_notification_history(
        &self,
    ) -> Result<Vec<(Uuid, i64, NaiveDateTime)>> {
        let mut conn = self.get_conn().await?;

        let results: Vec<(Uuid, i64, Option<NaiveDateTime>)> = user_room_bindings::table
            .filter(user_room_bindings::last_notified_at.is_not_null())
            .select((
                user_room_bindings::user_id,
                user_room_bindings::roomid,
                user_room_bindings::last_notified_at,
            ))
            .load(&mut conn)
            .await
            .map_err(AppError::Database)?;

        // 过滤掉 None 值（虽然 IS NOT NULL 已经过滤，但 Diesel 返回 Option）
        let filtered: Vec<(Uuid, i64, NaiveDateTime)> = results
            .into_iter()
            .filter_map(|(user_id, roomid, time_opt)| time_opt.map(|time| (user_id, roomid, time)))
            .collect();

        tracing::info!(count = filtered.len(), "加载通知历史记录");

        Ok(filtered)
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
