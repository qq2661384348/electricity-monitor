//! 通知服务
//!
//! 后台任务：定期查询send_flag为true的房间并发送通知
//! 应用限流防止过多并发影响主业务
//!
//! # 升级说明
//! - 集成QQClient发送实际通知
//! - 使用UserRoomBindingRepository查询绑定用户
//! - 并发发送通知（限制10个并发）
//! - 两级通知开关：全局send_flag + 个人notification_enabled

use futures::stream::{self, StreamExt};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::time::{interval, Duration, Instant};
use uuid::Uuid;

use crate::domain::models::Room;
use crate::domain::services::{
    NotificationCache, NotificationGate, RateLimitOperation, RateLimiter,
};
use crate::errors::{AppError, Result};
use crate::infrastructure::notification::MessageBuilder;
use crate::infrastructure::repositories::{
    RoomRepository, UserRepository, UserRoomBindingRepository,
};
use crate::infrastructure::QQClient;

/// 通知处理统计信息
#[derive(Debug, Clone)]
pub struct NotificationStats {
    /// 处理的房间总数
    pub total_rooms: usize,

    /// 处理的绑定总数
    pub total_bindings: usize,

    /// 成功发送的通知数
    pub total_sent: usize,

    /// 发送失败的通知数
    pub total_failed: usize,

    /// 缓存命中率（0.0-1.0）
    pub cache_hit_rate: f64,

    /// 处理总耗时（毫秒）
    pub duration_ms: u128,
}

impl NotificationStats {
    /// 创建空统计
    pub fn empty() -> Self {
        Self {
            total_rooms: 0,
            total_bindings: 0,
            total_sent: 0,
            total_failed: 0,
            cache_hit_rate: 0.0,
            duration_ms: 0,
        }
    }
}

/// 通知服务
pub struct NotificationService {
    /// Room仓储
    room_repository: RoomRepository,

    /// 用户仓储
    user_repository: UserRepository,

    /// 绑定仓储
    binding_repository: UserRoomBindingRepository,

    /// QQ客户端
    qq_client: Arc<QQClient>,

    /// 限流器
    rate_limiter: Arc<RateLimiter>,

    /// 内存缓存
    cache: Arc<NotificationCache>,

    /// 通知门控器（防抖+去重）
    gate: Arc<NotificationGate>,

    /// 查询间隔（秒）
    query_interval_secs: u64,

    /// 并发发送限制
    concurrent_limit: usize,
}

impl NotificationService {
    /// 创建新的通知服务
    ///
    /// # 参数
    /// - `room_repository`: Room仓储
    /// - `user_repository`: 用户仓储
    /// - `binding_repository`: 绑定仓储
    /// - `qq_client`: QQ客户端
    /// - `rate_limiter`: 限流器
    /// - `gate`: 通知门控器（防抖+去重）
    /// - `config`: 通知配置（包含查询间隔、并发限制等）
    ///
    /// # 设计理念
    /// 使用配置对象模式减少参数数量，提高代码可维护性
    pub fn new(
        room_repository: RoomRepository,
        user_repository: UserRepository,
        binding_repository: UserRoomBindingRepository,
        qq_client: Arc<QQClient>,
        rate_limiter: Arc<RateLimiter>,
        gate: Arc<NotificationGate>,
        config: &crate::config::NotificationConfig,
    ) -> Self {
        // 创建缓存（用户1000个，绑定500个，TTL=300秒）
        let cache = Arc::new(NotificationCache::new(Some(1000), Some(500), Some(300)));

        Self {
            room_repository,
            user_repository,
            binding_repository,
            qq_client,
            rate_limiter,
            cache,
            gate,
            query_interval_secs: config.query_interval_secs,
            concurrent_limit: config.concurrent_send_limit,
        }
    }

    /// 启动后台任务
    ///
    /// # 说明
    /// 此方法会启动一个Tokio任务，定期查询send_flag=true的房间
    /// 应用限流（每秒1次）防止过多并发
    ///
    /// # 工程化设计
    /// 使用标准Tokio提取字段模式，避免Arc<Self>生命周期问题
    pub fn spawn_worker(self) -> tokio::task::JoinHandle<()> {
        // 提取所有必要的字段（标准Tokio实践）
        let room_repository = self.room_repository;
        let user_repository = self.user_repository;
        let binding_repository = self.binding_repository;
        let qq_client = self.qq_client;
        let rate_limiter = self.rate_limiter;
        let cache = self.cache;
        let gate = self.gate;
        let query_interval_secs = self.query_interval_secs;
        let concurrent_limit = self.concurrent_limit;

        tokio::spawn(async move {
            tracing::info!("通知服务已启动，查询间隔: {}秒", query_interval_secs);

            let mut interval_timer = interval(Duration::from_secs(query_interval_secs));

            loop {
                // 等待下一个间隔
                interval_timer.tick().await;

                // 应用限流
                if let Err(e) = rate_limiter
                    .wait_for_rate_limit(RateLimitOperation::Query)
                    .await
                {
                    tracing::error!("限流检查失败: {}", e);
                    continue;
                }

                // Spawn独立任务处理通知（完全避免生命周期问题）
                // 每个任务拥有自己的Repository克隆
                let room_repo = room_repository.clone();
                let user_repo = user_repository.clone();
                let binding_repo = binding_repository.clone();
                let client = Arc::clone(&qq_client);
                let cache_clone = Arc::clone(&cache);
                let gate_clone = Arc::clone(&gate);

                let handle = tokio::spawn(async move {
                    Self::process_notifications_internal(
                        room_repo,
                        user_repo,
                        binding_repo,
                        client,
                        cache_clone,
                        gate_clone,
                        concurrent_limit,
                    )
                    .await
                });

                // 等待任务完成并处理错误（非阻塞其他循环）
                tokio::spawn(async move {
                    match handle.await {
                        Ok(Ok(())) => {
                            // 成功完成
                        }
                        Ok(Err(e)) => {
                            tracing::error!("处理通知失败: {}", e);
                        }
                        Err(e) => {
                            tracing::error!("通知任务panic: {}", e);
                        }
                    }
                });
            }
        })
    }

    /// 批量处理房间通知（核心共享逻辑）
    ///
    /// # 参数
    /// - `rooms`: 需要处理的房间列表
    /// - `user_repository`: 用户仓储
    /// - `binding_repository`: 绑定仓储
    /// - `qq_client`: QQ客户端
    /// - `cache`: 通知缓存
    /// - `concurrent_limit`: 并发限制
    ///
    /// # 返回
    /// 处理统计信息
    ///
    /// # 设计理念
    /// - 批量查询：3次数据库查询替代N+1
    /// - 内存缓存：LRU缓存减少重复查询
    /// - 并发发送：buffer_unordered提升吞吐量
    /// - 统一架构：定时任务和手动触发共享此逻辑
    async fn batch_process_room_notifications(
        rooms: Vec<Room>,
        user_repository: &UserRepository,
        binding_repository: &UserRoomBindingRepository,
        qq_client: &Arc<QQClient>,
        cache: &Arc<NotificationCache>,
        gate: &Arc<NotificationGate>,
        concurrent_limit: usize,
    ) -> Result<NotificationStats> {
        let start_time = Instant::now();

        if rooms.is_empty() {
            return Ok(NotificationStats::empty());
        }

        let room_count = rooms.len();
        tracing::info!("批量处理 {} 个房间通知", room_count);

        // 2. 提取所有roomid
        let roomids: Vec<i32> = rooms.iter().map(|r| r.roomid).collect();

        // 3. 批量查询所有房间的绑定关系（1次查询）
        let query_start = Instant::now();
        let all_bindings = binding_repository
            .find_active_bindings_by_roomids(&roomids)
            .await?;
        let query_duration = query_start.elapsed();

        if all_bindings.is_empty() {
            tracing::info!("所有房间都没有启用通知的绑定");
            return Ok(NotificationStats {
                total_rooms: room_count,
                total_bindings: 0,
                total_sent: 0,
                total_failed: 0,
                cache_hit_rate: 0.0,
                duration_ms: start_time.elapsed().as_millis(),
            });
        }

        tracing::info!(
            "查询到 {} 个绑定关系，耗时 {:?}",
            all_bindings.len(),
            query_duration
        );

        // 4. 提取唯一的user_id列表
        let user_ids: HashSet<Uuid> = all_bindings.iter().map(|b| b.user_id).collect();

        // 5. 批量查询所有用户（1次查询）
        let users_vec: Vec<Uuid> = user_ids.into_iter().collect();
        let users = user_repository.find_by_ids(&users_vec).await?;

        tracing::info!("批量查询到 {} 个用户", users.len());

        // 6. 构建用户Map（便于快速查找）
        let user_map: HashMap<Uuid, _> = users.into_iter().map(|u| (u.id, u)).collect();

        // 7. 更新缓存
        for user in user_map.values() {
            cache.set_user(user.clone()).await;
        }

        // 8. 按roomid分组绑定关系
        let mut bindings_by_room: HashMap<i32, Vec<_>> = HashMap::new();
        for binding in all_bindings {
            bindings_by_room
                .entry(binding.roomid)
                .or_default()
                .push(binding);
        }

        // 9. 更新绑定缓存
        cache.set_bindings_batch(bindings_by_room.clone()).await;

        // 10. 并发发送通知（使用内存数据，带持久化）
        let bindings_by_room = Arc::new(bindings_by_room);
        let user_map = Arc::new(user_map);

        let results: Vec<Result<usize>> = stream::iter(rooms)
            .map(|room| {
                let bindings_by_room = Arc::clone(&bindings_by_room);
                let user_map = Arc::clone(&user_map);
                let qq_client = Arc::clone(qq_client);
                let gate_clone = Arc::clone(gate);
                let binding_repo = binding_repository.clone();
                async move {
                    Self::send_room_notifications_optimized(
                        &room,
                        bindings_by_room.get(&room.roomid),
                        &user_map,
                        &qq_client,
                        &gate_clone,
                        &binding_repo,
                    )
                    .await
                }
            })
            .buffer_unordered(concurrent_limit)
            .collect()
            .await;

        // 11. 统计结果
        let mut total_sent = 0;
        let mut total_errors = 0;
        for result in results {
            match result {
                Ok(sent) => total_sent += sent,
                Err(e) => {
                    tracing::error!("房间通知失败: {}", e);
                    total_errors += 1;
                }
            }
        }

        let total_duration = start_time.elapsed();

        // 12. 打印缓存统计
        cache.log_stats().await;

        let stats = NotificationStats {
            total_rooms: room_count,
            total_bindings: bindings_by_room.len(),
            total_sent,
            total_failed: total_errors,
            cache_hit_rate: 0.0, // TODO: 从cache获取实际命中率
            duration_ms: total_duration.as_millis(),
        };

        tracing::info!(
            rooms = stats.total_rooms,
            bindings = stats.total_bindings,
            sent = stats.total_sent,
            failed = stats.total_failed,
            duration_ms = stats.duration_ms,
            "批量通知处理完成"
        );

        Ok(stats)
    }

    /// 查询并处理需要通知的房间（定时任务入口）
    ///
    /// # 工程化设计
    /// - 使用owned Repository（Clone）完全解决Send + 'static生命周期问题
    /// - 复用batch_process_room_notifications共享逻辑
    async fn process_notifications_internal(
        room_repository: RoomRepository,
        user_repository: UserRepository,
        binding_repository: UserRoomBindingRepository,
        qq_client: Arc<QQClient>,
        cache: Arc<NotificationCache>,
        gate: Arc<NotificationGate>,
        concurrent_limit: usize,
    ) -> Result<()> {
        tracing::debug!("开始查询需要通知的房间（定时任务）");

        // 1. 查询send_flag=true的房间（1次查询）
        let rooms = room_repository.find_rooms_with_send_flag_true().await?;

        if rooms.is_empty() {
            tracing::debug!("没有需要通知的房间");
            return Ok(());
        }

        tracing::info!("找到 {} 个需要通知的房间", rooms.len());

        // 2. 调用共享批量处理逻辑
        let _stats = Self::batch_process_room_notifications(
            rooms,
            &user_repository,
            &binding_repository,
            &qq_client,
            &cache,
            &gate,
            concurrent_limit,
        )
        .await?;

        Ok(())
    }

    /// 发送房间的所有通知（优化版：使用内存数据，带持久化）
    ///
    /// # 参数
    /// - `room`: 需要通知的房间
    /// - `bindings_opt`: 房间的绑定关系（已从内存获取）
    /// - `user_map`: 用户映射表（已从内存获取）
    /// - `binding_repository`: 绑定仓储（用于持久化通知状态）
    ///
    /// # 返回
    /// 成功发送的通知数量
    ///
    /// # 说明
    /// 此方法使用已经批量查询并缓存的数据，避免重复数据库查询
    /// 通知状态会持久化到数据库，防止重启后重复通知
    ///
    /// # 工程化设计
    /// 使用静态方法避免生命周期问题，明确传递所需参数
    async fn send_room_notifications_optimized(
        room: &Room,
        bindings_opt: Option<&Vec<crate::domain::models::UserRoomBinding>>,
        user_map: &HashMap<Uuid, crate::domain::models::User>,
        qq_client: &Arc<QQClient>,
        gate: &Arc<NotificationGate>,
        binding_repository: &UserRoomBindingRepository,
    ) -> Result<usize> {
        tracing::debug!(
            "开始处理房间通知（优化版）: room_id={}, roomid={}, room_name={}",
            room.id,
            room.roomid,
            room.room_name
        );

        // 获取绑定关系
        let bindings = match bindings_opt {
            Some(b) => b,
            None => {
                tracing::debug!("房间无启用通知的绑定: roomid={}", room.roomid);
                return Ok(0);
            }
        };

        if bindings.is_empty() {
            return Ok(0);
        }

        tracing::debug!(
            "房间有 {} 个启用通知的绑定: roomid={}",
            bindings.len(),
            room.roomid
        );

        // 并发发送通知给所有用户（使用内存中的用户数据，带持久化）
        // 工程化设计：克隆所有数据以完全避免生命周期问题
        let public_url = crate::config::AppConfig::global().public_site.public_url();
        let message = MessageBuilder::build_electricity_alert_message(room, &public_url);
        let user_map_arc = Arc::new(user_map.clone());
        let bindings_vec: Vec<_> = bindings.to_vec();

        let results: Vec<Result<()>> = stream::iter(bindings_vec)
            .map(|binding| {
                let qq_client = Arc::clone(qq_client);
                let message = message.clone();
                let user_map = Arc::clone(&user_map_arc);
                let gate_clone = Arc::clone(gate);
                let room_clone = room.clone();
                let binding_repo = binding_repository.clone();
                async move {
                    // 从内存Map中查找用户
                    let user = match user_map.get(&binding.user_id) {
                        Some(u) => u,
                        None => {
                            tracing::warn!(
                                "用户数据未找到: user_id={}, binding_id={}",
                                binding.user_id,
                                binding.id
                            );
                            return Ok::<(), AppError>(());
                        }
                    };

                    // 检查用户激活状态
                    if !user.is_active {
                        tracing::debug!(
                            "用户已停用，跳过通知: user_id={}, qq_number={}",
                            user.id,
                            user.qq_number
                        );
                        return Ok(());
                    }

                    // ⭐ 门控检查：防抖+去重
                    if !gate_clone.should_notify(user.id, &room_clone).await {
                        tracing::debug!(
                            user_id = %user.id,
                            roomid = room_clone.roomid,
                            "通知被门控阻止（等待恢复观察期或已发送）"
                        );
                        return Ok(());
                    }

                    // 发送通知
                    if let Err(e) = qq_client
                        .send_private_message(&user.qq_number, &message)
                        .await
                    {
                        tracing::error!("发送通知失败: qq_number={}, error={}", user.qq_number, e);
                        return Err(AppError::Internal(format!("发送通知失败: {}", e)));
                    }

                    // ⭐ 标记已发送（持久化到数据库）
                    if let Err(e) = gate_clone
                        .mark_notified_persistent(user.id, room_clone.roomid, &binding_repo)
                        .await
                    {
                        // 持久化失败不影响通知发送，仅记录警告
                        tracing::warn!(
                            user_id = %user.id,
                            roomid = room_clone.roomid,
                            error = %e,
                            "标记通知状态持久化失败，内存状态已更新"
                        );
                    }

                    tracing::info!(
                        user_id = %user.id,
                        roomid = room_clone.roomid,
                        qq_number = &user.qq_number,
                        "通知发送成功（已持久化）"
                    );
                    Ok(())
                }
            })
            .buffer_unordered(5) // 同一房间内并发发送给5个用户
            .collect()
            .await;

        // 统计成功发送的数量
        let sent_count = results.iter().filter(|r| r.is_ok()).count();

        tracing::debug!(
            roomid = room.roomid,
            total = results.len(),
            sent = sent_count,
            failed = results.len() - sent_count,
            "房间通知发送完成（优化版）"
        );

        Ok(sent_count)
    }

    /// 手动触发通知检查（供API调用）
    ///
    /// # 返回
    /// 处理统计信息
    ///
    /// # 工程化设计
    /// - 复用batch_process_room_notifications核心逻辑
    /// - 统一批量查询架构，避免N+1问题
    /// - 返回详细统计信息供API响应
    pub async fn trigger_manual_check(&self) -> Result<NotificationStats> {
        tracing::info!("手动触发通知检查（批量优化版）");

        // 1. 查询需要通知的房间
        let rooms = self
            .room_repository
            .find_rooms_with_send_flag_true()
            .await?;

        if rooms.is_empty() {
            tracing::info!("没有需要通知的房间");
            return Ok(NotificationStats::empty());
        }

        tracing::info!("找到 {} 个需要通知的房间", rooms.len());

        // 2. 调用批量处理逻辑（与定时任务共享）
        Self::batch_process_room_notifications(
            rooms,
            &self.user_repository,
            &self.binding_repository,
            &self.qq_client,
            &self.cache,
            &self.gate,
            self.concurrent_limit,
        )
        .await
    }
}

///
/// # 注意
/// 此trait是公开接口，供外部模块扩展使用
/// 使用显式Future返回类型以支持Send bound
pub trait NotificationSender: Send + Sync {
    /// 发送电费超限通知
    ///
    /// # 参数
    /// - `room`: 房间信息
    ///
    /// # 返回
    /// 是否发送成功的Future
    fn send_electricity_alert(
        &self,
        room: &Room,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;
}

/// 模拟通知发送器（用于测试）
#[cfg(test)]
pub struct MockNotificationSender;

#[cfg(test)]
impl NotificationSender for MockNotificationSender {
    async fn send_electricity_alert(&self, room: &Room) -> Result<bool> {
        println!("发送通知: room_id={}, roomid={}", room.id, room.roomid);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_mock_notification_sender() {
        // 测试模拟通知发送器
    }
}
