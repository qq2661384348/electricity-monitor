//! User数据仓储实现
//!
//! 提供User实体的数据访问操作

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::domain::models::{NewUser, UpdateUserRole, User};
use crate::errors::{AppError, Result};
use crate::infrastructure::database::schema::users;
use crate::infrastructure::DbPool;

/// User数据仓储
#[derive(Clone)]
pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
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

    /// 根据QQ号查找用户
    ///
    /// # 参数
    /// - `qq_number`: QQ号
    ///
    /// # 返回
    /// - `Some(User)`: 找到用户
    /// - `None`: 用户不存在
    pub async fn find_by_qq_number(&self, qq_number: &str) -> Result<Option<User>> {
        let mut conn = self.get_conn().await?;

        users::table
            .filter(users::qq_number.eq(qq_number))
            .select(User::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(AppError::Database)
    }

    /// 根据UUID查找用户
    ///
    /// # 参数
    /// - `id`: 用户UUID
    ///
    /// # 返回
    /// - `Some(User)`: 找到用户
    /// - `None`: 用户不存在
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let mut conn = self.get_conn().await?;

        users::table
            .find(id)
            .select(User::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(AppError::Database)
    }

    /// 创建用户或查找已存在的用户
    ///
    /// # 参数
    /// - `qq_number`: QQ号
    /// - `role`: 用户角色（默认: "user"）
    ///
    /// # 返回
    /// 用户实体（已存在或新创建）
    ///
    /// # 说明
    /// 使用INSERT ... ON CONFLICT DO NOTHING避免竞态条件
    /// 如果插入失败（用户已存在），则查询返回
    pub async fn create_or_find(&self, qq_number: &str, role: &str) -> Result<User> {
        let mut conn = self.get_conn().await?;

        let new_user = NewUser {
            qq_number: qq_number.to_string(),
            role: role.to_string(),
            is_active: true,
        };

        // 尝试插入，如果冲突则忽略
        let insert_result = diesel::insert_into(users::table)
            .values(&new_user)
            .on_conflict(users::qq_number)
            .do_nothing()
            .execute(&mut conn)
            .await
            .map_err(AppError::Database)?;

        // 如果插入成功（affected_rows > 0），查询刚插入的用户
        // 如果插入失败（冲突），查询已存在的用户
        let user = users::table
            .filter(users::qq_number.eq(qq_number))
            .select(User::as_select())
            .first(&mut conn)
            .await
            .map_err(AppError::Database)?;

        if insert_result > 0 {
            tracing::info!(
                qq_number = qq_number,
                user_id = %user.id,
                role = role,
                "创建新用户成功"
            );
        } else {
            tracing::debug!(
                qq_number = qq_number,
                user_id = %user.id,
                "用户已存在"
            );
        }

        Ok(user)
    }

    /// 更新用户角色
    ///
    /// # 参数
    /// - `id`: 用户UUID
    /// - `new_role`: 新角色
    ///
    /// # 返回
    /// 更新后的User实体
    pub async fn update_role(&self, id: Uuid, new_role: &str) -> Result<User> {
        let mut conn = self.get_conn().await?;

        let update = UpdateUserRole {
            role: new_role.to_string(),
        };

        let user = diesel::update(users::table.find(id))
            .set(&update)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(AppError::Database)?;

        tracing::info!(
            user_id = %id,
            new_role = new_role,
            "更新用户角色成功"
        );

        Ok(user)
    }

    /// 更新用户激活状态
    ///
    /// # 参数
    /// - `id`: 用户UUID
    /// - `is_active`: 是否激活
    ///
    /// # 返回
    /// 更新的行数
    pub async fn update_active_status(&self, id: Uuid, is_active: bool) -> Result<usize> {
        let mut conn = self.get_conn().await?;

        let affected_rows = diesel::update(users::table.find(id))
            .set(users::is_active.eq(is_active))
            .execute(&mut conn)
            .await
            .map_err(AppError::Database)?;

        tracing::info!(
            user_id = %id,
            is_active = is_active,
            affected_rows = affected_rows,
            "更新用户激活状态"
        );

        if affected_rows == 0 {
            tracing::warn!(
                user_id = %id,
                "更新用户激活状态但没有匹配的用户"
            );
        }

        Ok(affected_rows)
    }

    /// 查询所有激活的用户
    ///
    /// # 返回
    /// 激活用户列表
    pub async fn find_all_active(&self) -> Result<Vec<User>> {
        let mut conn = self.get_conn().await?;

        users::table
            .filter(users::is_active.eq(true))
            .select(User::as_select())
            .order_by(users::created_at.desc())
            .load(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 查询管理员用户列表
    ///
    /// # 返回
    /// 管理员用户列表
    pub async fn find_all_admins(&self) -> Result<Vec<User>> {
        let mut conn = self.get_conn().await?;

        users::table
            .filter(users::role.eq("admin"))
            .filter(users::is_active.eq(true))
            .select(User::as_select())
            .order_by(users::created_at.asc())
            .load(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 批量查询用户（根据UUID列表）
    ///
    /// # 参数
    /// - `user_ids`: 用户UUID切片
    ///
    /// # 返回
    /// 用户列表（注意：返回顺序可能与输入顺序不同）
    ///
    /// # 说明
    /// - 使用 `IN (...)` 批量查询，性能优于逐个查询
    /// - 自动分批处理（每批1000个ID）防止SQL过长
    /// - 返回的用户数可能少于输入ID数（不存在的ID会被忽略）
    ///
    /// # 示例
    /// ```ignore
    /// let user_ids = vec![uuid1, uuid2, uuid3];
    /// let users = repo.find_by_ids(&user_ids).await?;
    /// // 将结果转换为HashMap便于快速查找
    /// let user_map: HashMap<Uuid, User> = users.into_iter()
    ///     .map(|u| (u.id, u))
    ///     .collect();
    /// ```
    pub async fn find_by_ids(&self, user_ids: &[Uuid]) -> Result<Vec<User>> {
        if user_ids.is_empty() {
            return Ok(Vec::new());
        }

        // 分批查询（每批最多1000个ID）
        const BATCH_SIZE: usize = 1000;
        let mut all_users = Vec::with_capacity(user_ids.len());

        for chunk in user_ids.chunks(BATCH_SIZE) {
            let mut conn = self.get_conn().await?;

            let users = users::table
                .filter(users::id.eq_any(chunk))
                .select(User::as_select())
                .load(&mut conn)
                .await
                .map_err(AppError::Database)?;

            all_users.extend(users);
        }

        tracing::debug!(
            requested = user_ids.len(),
            found = all_users.len(),
            batches = user_ids.len().div_ceil(BATCH_SIZE),
            "批量查询用户完成"
        );

        Ok(all_users)
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
