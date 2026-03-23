//! 电费历史记录仓储

use crate::domain::models::{ElectricityHistory, NewElectricityHistory};
use crate::errors::{AppError, Result};
use crate::infrastructure::{database::schema::*, DbPool};
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

/// 电费历史记录仓储
pub struct ElectricityHistoryRepository {
    pool: DbPool,
}

impl ElectricityHistoryRepository {
    /// 创建仓储实例
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 获取数据库连接（内部辅助方法）
    async fn get_conn(
        &self,
    ) -> Result<diesel_async::pooled_connection::deadpool::Object<diesel_async::AsyncPgConnection>>
    {
        self.pool.get().await.map_err(|e| {
            tracing::error!("Failed to get database connection: {}", e);
            AppError::Internal(format!("Failed to get database connection: {}", e))
        })
    }

    /// 批量插入历史记录（从rooms表）
    ///
    /// # 返回
    /// 插入的行数
    ///
    /// # 说明
    /// - 从rooms表复制当前的roomid和electricity_fee
    /// - recorded_at使用当前时间
    /// - 只复制is_active=true的房间
    pub async fn batch_insert_from_rooms(&self) -> Result<usize> {
        let mut conn = self.get_conn().await?;
        let now = Utc::now().naive_utc();

        // 查询所有活跃房间的roomid和electricity_fee
        let room_data: Vec<(i32, f32)> = rooms::table
            .filter(rooms::is_active.eq(true))
            .select((rooms::roomid, rooms::electricity_fee))
            .load(&mut conn)
            .await
            .map_err(AppError::Database)?;

        if room_data.is_empty() {
            tracing::info!("批量插入历史记录：无活跃房间");
            return Ok(0);
        }

        // 构造历史记录
        let histories: Vec<NewElectricityHistory> = room_data
            .into_iter()
            .map(|(roomid, fee)| NewElectricityHistory::new(roomid, fee, now))
            .collect();

        let count = histories.len();

        // 批量插入
        diesel::insert_into(electricity_history::table)
            .values(&histories)
            .execute(&mut conn)
            .await
            .map_err(AppError::Database)?;

        tracing::info!(
            count = count,
            recorded_at = %now,
            "批量插入历史记录完成"
        );

        Ok(count)
    }

    /// 删除旧历史记录
    ///
    /// # 参数
    /// - `days`: 保留天数（删除>days天前的数据）
    ///
    /// # 返回
    /// 删除的行数
    ///
    /// # 说明
    /// - 删除recorded_at < (now - days)的记录
    /// - 使用索引优化查询性能
    pub async fn delete_old_records(&self, days: i64) -> Result<usize> {
        let mut conn = self.get_conn().await?;
        let cutoff_time = Utc::now().naive_utc() - chrono::Duration::days(days);

        let deleted = diesel::delete(
            electricity_history::table.filter(electricity_history::recorded_at.lt(cutoff_time)),
        )
        .execute(&mut conn)
        .await
        .map_err(AppError::Database)?;

        tracing::info!(
            deleted = deleted,
            days = days,
            cutoff_time = %cutoff_time,
            "删除旧历史记录完成"
        );

        Ok(deleted)
    }

    /// 查询房间历史记录
    ///
    /// # 参数
    /// - `roomid`: 房间ID
    /// - `limit`: 查询数量限制
    ///
    /// # 返回
    /// 历史记录列表（按时间降序）
    ///
    /// # 说明
    /// - 使用复合索引（roomid, recorded_at DESC）优化查询
    /// - 默认返回最近的N条记录
    pub async fn find_by_roomid(&self, roomid: i32, limit: i64) -> Result<Vec<ElectricityHistory>> {
        let mut conn = self.get_conn().await?;

        electricity_history::table
            .filter(electricity_history::roomid.eq(roomid))
            .order_by(electricity_history::recorded_at.desc())
            .limit(limit)
            .select(ElectricityHistory::as_select())
            .load(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 查询指定时间范围的历史记录
    ///
    /// # 参数
    /// - `roomid`: 房间ID
    /// - `start`: 开始时间
    /// - `end`: 结束时间
    ///
    /// # 返回
    /// 历史记录列表（按时间升序）
    pub async fn find_by_roomid_and_time_range(
        &self,
        roomid: i32,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> Result<Vec<ElectricityHistory>> {
        let mut conn = self.get_conn().await?;

        electricity_history::table
            .filter(
                electricity_history::roomid
                    .eq(roomid)
                    .and(electricity_history::recorded_at.ge(start))
                    .and(electricity_history::recorded_at.le(end)),
            )
            .order_by(electricity_history::recorded_at.asc())
            .select(ElectricityHistory::as_select())
            .load(&mut conn)
            .await
            .map_err(AppError::Database)
    }

    /// 统计历史记录总数
    ///
    /// # 返回
    /// 总记录数
    pub async fn count_all(&self) -> Result<i64> {
        let mut conn = self.get_conn().await?;

        electricity_history::table
            .count()
            .get_result(&mut conn)
            .await
            .map_err(AppError::Database)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_struct() {
        // 测试结构体大小（编译时验证）
        assert_eq!(std::mem::size_of::<ElectricityHistoryRepository>(), 8);
    }
}
