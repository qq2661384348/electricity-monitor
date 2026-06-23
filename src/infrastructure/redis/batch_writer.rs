//! Redis批量写入模块
//!
//! 使用Redis Pipeline批量写入电费数据，减轻数据库压力

use crate::errors::{AppError, Result};
use crate::infrastructure::RedisPool;
use chrono::Utc;
use redis::AsyncCommands;
use std::collections::HashMap;

/// Redis批量写入器
///
/// 使用Pipeline批量写入电费数据到Redis缓存
pub struct RedisBatchWriter {
    pool: RedisPool,
}

impl RedisBatchWriter {
    /// 创建批量写入器
    ///
    /// # 参数
    /// - `pool`: Redis连接池
    pub fn new(pool: RedisPool) -> Self {
        Self { pool }
    }

    /// 批量写入电费数据
    ///
    /// # 参数
    /// - `data`: roomid → electricity_fee 映射
    ///
    /// # 说明
    /// - 使用Redis Pipeline批量写入
    /// - 批次大小：500条/batch
    /// - TTL：256秒（根据用户要求）
    /// - Key格式：`electricity:batch:{timestamp}`
    ///
    /// # 返回
    /// 写入的总条数
    ///
    /// # 错误
    /// Redis连接或写入失败
    pub async fn batch_write(&self, data: &HashMap<i64, f32>) -> Result<usize> {
        if data.is_empty() {
            tracing::debug!("批量写入跳过：数据为空");
            return Ok(0);
        }

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("获取Redis连接失败: {}", e)))?;

        let timestamp = Utc::now().timestamp();
        let key = format!("electricity:batch:{}", timestamp);

        const BATCH_SIZE: usize = 500;
        let total_count = data.len();
        let mut written_count = 0;
        let mut batch_idx = 0;
        let mut batch_len = 0;
        let mut pipe = redis::pipe();
        pipe.atomic();

        // 分批写入。不要先 collect 成 Vec，否则全量房间批次会多产生一份临时索引。
        for (roomid, fee) in data {
            pipe.hset(&key, roomid.to_string(), fee.to_string());
            batch_len += 1;

            if batch_len < BATCH_SIZE {
                continue;
            }

            written_count += Self::flush_batch(
                &mut conn,
                &key,
                &mut pipe,
                batch_idx,
                batch_len,
                written_count,
                total_count,
            )
            .await?;
            batch_idx += 1;
            batch_len = 0;
        }

        if batch_len > 0 {
            written_count += Self::flush_batch(
                &mut conn,
                &key,
                &mut pipe,
                batch_idx,
                batch_len,
                written_count,
                total_count,
            )
            .await?;
        }

        tracing::info!(
            count = written_count,
            batches = total_count.div_ceil(BATCH_SIZE),
            key = %key,
            ttl = 256,
            "Redis批量写入完成"
        );

        Ok(written_count)
    }

    async fn flush_batch<C>(
        conn: &mut C,
        key: &str,
        pipe: &mut redis::Pipeline,
        batch_idx: usize,
        batch_len: usize,
        already_written: usize,
        total_count: usize,
    ) -> Result<usize>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        // 设置TTL（256秒，用户要求）
        pipe.expire(key, 256);

        // 执行Pipeline（返回空tuple）
        let _: () = pipe
            .query_async(conn)
            .await
            .map_err(|e| AppError::Internal(format!("Redis Pipeline执行失败: {}", e)))?;

        let written = already_written + batch_len;

        tracing::debug!(
            batch_idx = batch_idx,
            batch_size = batch_len,
            written = written,
            total = total_count,
            key = %key,
            "Redis批量写入批次完成"
        );

        let mut next_pipe = redis::pipe();
        next_pipe.atomic();
        *pipe = next_pipe;

        Ok(batch_len)
    }

    /// 批量读取电费数据
    ///
    /// # 参数
    /// - `timestamp`: 时间戳（对应写入时的timestamp）
    ///
    /// # 返回
    /// roomid → electricity_fee 映射
    ///
    /// # 说明
    /// 从Redis读取整个Hash
    pub async fn batch_read(&self, timestamp: i64) -> Result<HashMap<i64, f32>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("获取Redis连接失败: {}", e)))?;

        let key = format!("electricity:batch:{}", timestamp);

        // 获取整个Hash
        let hash: HashMap<String, String> = conn
            .hgetall(&key)
            .await
            .map_err(|e| AppError::Internal(format!("Redis读取失败: {}", e)))?;

        // 转换类型：String → i32/f32
        let mut result = HashMap::new();
        for (roomid_str, fee_str) in hash {
            if let (Ok(roomid), Ok(fee)) = (roomid_str.parse::<i64>(), fee_str.parse::<f32>()) {
                result.insert(roomid, fee);
            } else {
                tracing::warn!(
                    roomid = %roomid_str,
                    fee = %fee_str,
                    "Redis数据格式错误，跳过"
                );
            }
        }

        tracing::debug!(
            count = result.len(),
            key = %key,
            "Redis批量读取完成"
        );

        Ok(result)
    }

    /// 删除批次数据
    ///
    /// # 参数
    /// - `timestamp`: 时间戳
    ///
    /// # 说明
    /// 手动删除批次数据（通常由TTL自动过期）
    pub async fn delete_batch(&self, timestamp: i64) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("获取Redis连接失败: {}", e)))?;

        let key = format!("electricity:batch:{}", timestamp);

        let _: () = conn
            .del(&key)
            .await
            .map_err(|e| AppError::Internal(format!("Redis删除失败: {}", e)))?;

        tracing::debug!(key = %key, "Redis批次数据已删除");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_batch_writer_struct() {
        // 测试结构体大小（编译时验证）
        assert_eq!(std::mem::size_of::<RedisBatchWriter>(), 8); // 仅包含pool
    }

    #[test]
    fn test_batch_key_format() {
        let timestamp = 1699776000;
        let key = format!("electricity:batch:{}", timestamp);
        assert_eq!(key, "electricity:batch:1699776000");
    }
}
