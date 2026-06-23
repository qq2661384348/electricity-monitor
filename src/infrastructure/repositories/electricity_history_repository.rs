//! 电费历史记录仓储

use crate::domain::models::ElectricityHistory;
use crate::errors::{AppError, Result};
use crate::infrastructure::{database::schema::*, DbPool};
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::Timestamp;
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

        // 生产库每小时会为数千个房间写入一次历史快照。这里必须让数据库
        // 直接执行 INSERT ... SELECT，避免把所有房间先 load 到 Rust 堆、
        // 再构造逐条历史记录和超大 INSERT 语句导致容器 RSS 高水位。
        let count = diesel::sql_query(
            "INSERT INTO electricity_history (roomid, electricity_fee, recorded_at)
             SELECT roomid, electricity_fee, $1
             FROM rooms
             WHERE is_active = TRUE",
        )
        .bind::<Timestamp, _>(now)
        .execute(&mut conn)
        .await
        .map_err(AppError::Database)?;

        if count == 0 {
            tracing::info!("批量插入历史记录：无活跃房间");
            return Ok(0);
        }

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
    pub async fn find_by_roomid(&self, roomid: i64, limit: i64) -> Result<Vec<ElectricityHistory>> {
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
        roomid: i64,
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
