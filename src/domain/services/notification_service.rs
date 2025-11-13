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

use std::sync::Arc;
use tokio::time::{Duration, interval};
use futures::stream::{self, StreamExt};

use crate::domain::models::Room;
use crate::domain::services::{RateLimitOperation, RateLimiter};
use crate::errors::{AppError, Result};
use crate::infrastructure::repositories::{RoomRepository, UserRepository, UserRoomBindingRepository};
use crate::infrastructure::QQClient;
use crate::infrastructure::notification::MessageBuilder;

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
    /// - `query_interval_secs`: 查询间隔（秒），默认60秒
    /// - `concurrent_limit`: 并发发送限制，默认10
    pub fn new(
        room_repository: RoomRepository,
        user_repository: UserRepository,
        binding_repository: UserRoomBindingRepository,
        qq_client: Arc<QQClient>,
        rate_limiter: Arc<RateLimiter>,
        query_interval_secs: Option<u64>,
        concurrent_limit: Option<usize>,
    ) -> Self {
        Self {
            room_repository,
            user_repository,
            binding_repository,
            qq_client,
            rate_limiter,
            query_interval_secs: query_interval_secs.unwrap_or(60),
            concurrent_limit: concurrent_limit.unwrap_or(10),
        }
    }

    /// 启动后台任务
    /// 
    /// # 说明
    /// 此方法会启动一个Tokio任务，定期查询send_flag=true的房间
    /// 应用限流（每秒1次）防止过多并发
    pub fn spawn_worker(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            tracing::info!(
                "通知服务已启动，查询间隔: {}秒",
                self.query_interval_secs
            );
            
            let mut interval_timer = interval(Duration::from_secs(self.query_interval_secs));
            
            loop {
                // 等待下一个间隔
                interval_timer.tick().await;
                
                // 应用限流
                if let Err(e) = self.rate_limiter
                    .wait_for_rate_limit(RateLimitOperation::Query)
                    .await
                {
                    tracing::error!("限流检查失败: {}", e);
                    continue;
                }

                // 查询并处理需要通知的房间
                if let Err(e) = self.process_notifications().await {
                    tracing::error!("处理通知失败: {}", e);
                }
            }
        })
    }

    /// 查询并处理需要通知的房间
    async fn process_notifications(&self) -> Result<()> {
        tracing::debug!("开始查询需要通知的房间");

        // 1. 查询send_flag=true的房间
        let rooms = self.room_repository.find_rooms_with_send_flag_true().await?;

        if rooms.is_empty() {
            tracing::debug!("没有需要通知的房间");
            return Ok(());
        }

        tracing::info!("找到 {} 个需要通知的房间", rooms.len());

        // 2. 并发处理每个房间（限制并发数）
        let results: Vec<Result<usize>> = stream::iter(rooms)
            .map(|room| async move {
                self.send_room_notifications(&room).await
            })
            .buffer_unordered(self.concurrent_limit)
            .collect()
            .await;

        // 3. 统计结果
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

        tracing::info!(
            "通知处理完成: 成功={}, 失败={}",
            total_sent,
            total_errors
        );

        Ok(())
    }

    /// 发送房间的所有通知
    /// 
    /// # 参数
    /// - `room`: 需要通知的房间
    /// 
    /// # 返回
    /// 成功发送的通知数量
    /// 
    /// # 说明
    /// 1. 查询该房间启用通知的所有绑定
    /// 2. 查询每个绑定用户的QQ号
    /// 3. 并发发送通知
    async fn send_room_notifications(&self, room: &Room) -> Result<usize> {
        tracing::debug!(
            "开始处理房间通知: room_id={}, roomid={}, room_name={}",
            room.id,
            room.roomid,
            room.room_name
        );

        // 1. 查询启用通知的绑定
        let bindings = self.binding_repository
            .find_active_bindings_by_roomid(room.roomid)
            .await?;

        if bindings.is_empty() {
            tracing::debug!(
                "房间无启用通知的绑定: roomid={}",
                room.roomid
            );
            return Ok(0);
        }

        tracing::info!(
            "房间有 {} 个启用通知的绑定: roomid={}",
            bindings.len(),
            room.roomid
        );

        // 2. 并发发送通知给所有用户
        let results: Vec<Result<()>> = stream::iter(bindings)
            .map(|binding| async move {
                // 查询用户
                let user = match self.user_repository.find_by_id(binding.user_id).await? {
                    Some(u) => u,
                    None => {
                        tracing::warn!(
                            "绑定的用户不存在: user_id={}, binding_id={}",
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

                // 发送通知
                if let Err(e) = self.send_notification_to_user(&user.qq_number, room).await {
                    tracing::error!(
                        "发送通知失败: qq_number={}, roomid={}, error={}",
                        user.qq_number,
                        room.roomid,
                        e
                    );
                    return Err(e);
                }
                
                tracing::info!(
                    "通知发送成功: qq_number={}, roomid={}",
                    user.qq_number,
                    room.roomid
                );
                Ok(())
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
            "房间通知发送完成"
        );

        Ok(sent_count)
    }

    /// 发送通知给单个用户
    /// 
    /// # 参数
    /// - `qq_number`: 用户QQ号
    /// - `room`: 房间信息
    async fn send_notification_to_user(&self, qq_number: &str, room: &Room) -> Result<()> {
        // 构建通知消息
        let message = MessageBuilder::build_electricity_alert_message(room);

        // 发送通知
        self.qq_client
            .send_private_message(qq_number, &message)
            .await
            .map_err(|e| AppError::Internal(format!("QQ通知发送失败: {}", e)))?;

        Ok(())
    }

    /// 手动触发通知检查（供API调用）
    /// 
    /// # 返回
    /// 处理的房间数量
    pub async fn trigger_manual_check(&self) -> Result<usize> {
        tracing::info!("手动触发通知检查");

        // 查询需要通知的房间
        let rooms = self.room_repository.find_rooms_with_send_flag_true().await?;

        let mut total_sent = 0;
        for room in &rooms {
            match self.send_room_notifications(room).await {
                Ok(sent) => total_sent += sent,
                Err(e) => tracing::error!("手动通知发送失败: {}", e),
            }
        }

        Ok(total_sent)
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
    fn send_electricity_alert(&self, room: &Room) -> impl std::future::Future<Output = Result<bool>> + Send;
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
