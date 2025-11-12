//! 电费获取服务
//!
//! 整合所有组件，提供完整的电费获取和历史记录功能

use crate::domain::services::RoomIdCache;
use crate::errors::Result;
use crate::infrastructure::{
    electricity::RoomBatchFetcher,
    redis::RedisBatchWriter,
    repositories::{ElectricityHistoryRepository, RoomRepository},
    DbPool, RedisPool,
};
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

/// 电费获取服务
///
/// 核心服务，整合所有组件完成电费获取和更新
pub struct ElectricityFetcherService {
    /// RoomId缓存
    cache: Arc<RoomIdCache>,
    /// 批量获取器
    fetcher: Arc<RoomBatchFetcher>,
    /// Redis批量写入器
    redis_writer: Arc<RedisBatchWriter>,
    /// 房间仓储
    room_repo: RoomRepository,
    /// 历史记录仓储
    history_repo: ElectricityHistoryRepository,
}

/// 执行统计信息
#[derive(Debug, Clone)]
pub struct FetchStatistics {
    /// 成功获取数量
    pub success_count: usize,
    /// 失败数量
    pub failure_count: usize,
    /// 更新数据库的数量
    pub updated_count: usize,
    /// 执行时长（毫秒）
    pub duration_ms: u128,
}

impl ElectricityFetcherService {
    /// 创建服务实例
    ///
    /// # 参数
    /// - `api_url`: 电费API的URL（必须以?roomid=结尾）
    /// - `db_pool`: 数据库连接池
    /// - `redis_pool`: Redis连接池
    ///
    /// # 返回
    /// 服务实例
    ///
    /// # 错误
    /// - URL格式错误
    /// - 初始化缓存失败
    pub async fn new(
        api_url: String,
        db_pool: DbPool,
        redis_pool: RedisPool,
    ) -> Result<Self> {
        // 创建RoomId缓存
        let cache = Arc::new(RoomIdCache::new(db_pool.clone()).await?);

        // 创建批量获取器（50并发）
        let fetcher = Arc::new(
            RoomBatchFetcher::new(api_url, 50)
                .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?,
        );

        // 创建Redis批量写入器
        let redis_writer = Arc::new(RedisBatchWriter::new(redis_pool));

        // 创建仓储
        let room_repo = RoomRepository::new(db_pool.clone());
        let history_repo = ElectricityHistoryRepository::new(db_pool);

        Ok(Self {
            cache,
            fetcher,
            redis_writer,
            room_repo,
            history_repo,
        })
    }

    /// 执行电费获取任务
    ///
    /// # 工作流程
    /// 1. 从缓存获取roomid列表
    /// 2. 批量API获取（50并发）
    /// 3. 写入Redis缓存
    /// 4. 批量更新数据库
    ///
    /// # 返回
    /// 执行统计信息
    ///
    /// # 错误处理
    /// - API失败的房间记录DEBUG日志，不中断流程
    /// - 数据库失败返回错误
    pub async fn run_fetch_task(&self) -> Result<FetchStatistics> {
        let start_time = std::time::Instant::now();

        tracing::info!("开始执行电费获取任务");

        // 1. 获取roomid列表
        let room_ids = self.cache.get_all().await;
        let total_rooms = room_ids.len();

        if room_ids.is_empty() {
            tracing::warn!("无活跃房间，跳过电费获取");
            return Ok(FetchStatistics {
                success_count: 0,
                failure_count: 0,
                updated_count: 0,
                duration_ms: start_time.elapsed().as_millis(),
            });
        }

        tracing::info!(
            total_rooms = total_rooms,
            "开始批量获取电费"
        );

        // 2. 批量API获取（50并发）
        let fetch_result = self.fetcher.fetch_batch(room_ids.clone()).await;
        let success_count = fetch_result.len();
        let failure_count = total_rooms - success_count;

        tracing::info!(
            success = success_count,
            failure = failure_count,
            "批量获取电费完成"
        );

        if fetch_result.is_empty() {
            tracing::warn!("所有房间获取失败，跳过后续步骤");
            return Ok(FetchStatistics {
                success_count: 0,
                failure_count: total_rooms,
                updated_count: 0,
                duration_ms: start_time.elapsed().as_millis(),
            });
        }

        // 3. 写入Redis缓存（TTL=256s）
        self.redis_writer.batch_write(fetch_result.clone()).await?;

        // 4. 批量更新数据库（100条/batch）
        let updated_count = self
            .room_repo
            .batch_update_electricity_fee(fetch_result)
            .await?;

        let duration_ms = start_time.elapsed().as_millis();

        tracing::info!(
            success = success_count,
            failure = failure_count,
            updated = updated_count,
            duration_ms = duration_ms,
            "电费获取任务完成"
        );

        Ok(FetchStatistics {
            success_count,
            failure_count,
            updated_count,
            duration_ms,
        })
    }

    /// 执行历史记录任务
    ///
    /// # 工作流程
    /// 1. 批量插入当前电费到历史表
    /// 2. 删除8天前的历史数据
    ///
    /// # 返回
    /// (插入数量, 删除数量)
    ///
    /// # 说明
    /// - 通常每小时执行一次
    /// - 保留8天历史数据
    pub async fn run_history_task(&self) -> Result<(usize, usize)> {
        tracing::info!("开始执行历史记录任务");

        // 1. 批量插入当前电费
        let inserted = self.history_repo.batch_insert_from_rooms().await?;

        // 2. 删除8天前的数据
        let deleted = self.history_repo.delete_old_records(8).await?;

        tracing::info!(
            inserted = inserted,
            deleted = deleted,
            "历史记录任务完成"
        );

        Ok((inserted, deleted))
    }

    /// 刷新RoomId缓存
    ///
    /// # 说明
    /// - 从数据库重新加载活跃房间的roomid列表
    /// - 应在房间同步后调用
    pub async fn refresh_cache(&self) -> Result<()> {
        self.cache.refresh().await
    }

    /// 获取缓存大小
    ///
    /// # 返回
    /// 缓存中roomid的数量
    pub async fn cache_size(&self) -> usize {
        self.cache.len().await
    }

    /// 启动定时任务
    ///
    /// # 参数
    /// - `fetch_interval_minutes`: 电费获取间隔（分钟）
    /// - `history_interval_hours`: 历史记录间隔（小时）
    ///
    /// # 返回
    /// JobScheduler实例
    ///
    /// # 说明
    /// - 启动两个定时任务：电费获取 + 历史记录
    /// - 需要手动调用scheduler.start().await启动调度器
    pub async fn start_scheduler(
        service: Arc<ElectricityFetcherService>,
        fetch_interval_minutes: u64,
        history_interval_hours: u64,
    ) -> Result<JobScheduler> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| crate::errors::AppError::Internal(format!("创建调度器失败: {}", e)))?;

        // 任务1: 电费获取（每N分钟）
        let fetch_service = service.clone();
        let fetch_cron = format!("0 */{} * * * *", fetch_interval_minutes);
        let fetch_job = Job::new_async(fetch_cron.as_str(), move |_uuid, _lock| {
            let service = fetch_service.clone();
            Box::pin(async move {
                tracing::info!("开始定时电费获取任务");
                match service.run_fetch_task().await {
                    Ok(stats) => {
                        tracing::info!(
                            success = stats.success_count,
                            failure = stats.failure_count,
                            updated = stats.updated_count,
                            duration_ms = stats.duration_ms,
                            "定时电费获取任务完成"
                        );
                    }
                    Err(e) => {
                        tracing::error!("定时电费获取任务失败: {}", e);
                    }
                }
            })
        })
        .map_err(|e| crate::errors::AppError::Internal(format!("创建电费获取任务失败: {}", e)))?;

        scheduler
            .add(fetch_job)
            .await
            .map_err(|e| crate::errors::AppError::Internal(format!("添加电费获取任务失败: {}", e)))?;

        // 任务2: 历史记录（每N小时）
        let history_service = service.clone();
        let history_cron = format!("0 0 */{} * * *", history_interval_hours);
        let history_job = Job::new_async(history_cron.as_str(), move |_uuid, _lock| {
            let service = history_service.clone();
            Box::pin(async move {
                tracing::info!("开始定时历史记录任务");
                match service.run_history_task().await {
                    Ok((inserted, deleted)) => {
                        tracing::info!(
                            inserted = inserted,
                            deleted = deleted,
                            "定时历史记录任务完成"
                        );
                    }
                    Err(e) => {
                        tracing::error!("定时历史记录任务失败: {}", e);
                    }
                }
            })
        })
        .map_err(|e| crate::errors::AppError::Internal(format!("创建历史记录任务失败: {}", e)))?;

        scheduler
            .add(history_job)
            .await
            .map_err(|e| crate::errors::AppError::Internal(format!("添加历史记录任务失败: {}", e)))?;

        tracing::info!(
            fetch_interval = fetch_interval_minutes,
            history_interval = history_interval_hours,
            "电费获取定时任务已配置"
        );

        Ok(scheduler)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_statistics() {
        let stats = FetchStatistics {
            success_count: 100,
            failure_count: 5,
            updated_count: 98,
            duration_ms: 5000,
        };

        assert_eq!(stats.success_count, 100);
        assert_eq!(stats.failure_count, 5);
        assert_eq!(stats.updated_count, 98);
        assert_eq!(stats.duration_ms, 5000);
    }
}
