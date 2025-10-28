//! 通知服务
//! 
//! 后台任务：定期查询send_flag为true的房间并发送通知
//! 应用限流防止过多并发影响主业务

use std::sync::Arc;
use tokio::time::{sleep, Duration, interval};

use crate::domain::models::Room;
use crate::domain::services::{RateLimitOperation, RateLimiter};
use crate::errors::Result;
use crate::infrastructure::repositories::RoomRepository;

/// 通知服务
pub struct NotificationService {
    /// Room仓储
    repository: RoomRepository,
    
    /// 限流器
    rate_limiter: Arc<RateLimiter>,
    
    /// 查询间隔（秒）
    query_interval_secs: u64,
}

impl NotificationService {
    /// 创建新的通知服务
    /// 
    /// # 参数
    /// - `repository`: Room仓储
    /// - `rate_limiter`: 限流器
    /// - `query_interval_secs`: 查询间隔（秒），默认60秒
    pub fn new(
        repository: RoomRepository,
        rate_limiter: Arc<RateLimiter>,
        query_interval_secs: Option<u64>,
    ) -> Self {
        Self {
            repository,
            rate_limiter,
            query_interval_secs: query_interval_secs.unwrap_or(60),
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

        // 查询send_flag=true的房间
        let rooms = self.repository.find_rooms_with_send_flag_true().await?;

        if rooms.is_empty() {
            tracing::debug!("没有需要通知的房间");
            return Ok(());
        }

        tracing::info!("找到 {} 个需要通知的房间", rooms.len());

        // 处理每个房间
        for room in rooms {
            if let Err(e) = self.send_notification(&room).await {
                tracing::error!(
                    "发送通知失败: room_id={}, roomid={}, error={}",
                    room.id,
                    room.roomid,
                    e
                );
            }
        }

        Ok(())
    }

    /// 发送通知
    /// 
    /// # 参数
    /// - `room`: 需要通知的房间
    /// 
    /// # 说明
    /// 调用通知模块发送通知（预留接口）
    async fn send_notification(&self, room: &Room) -> Result<()> {
        tracing::info!(
            "发送通知: room_id={}, roomid={}, room_name={}, electricity_fee={}, threshold={}",
            room.id,
            room.roomid,
            room.room_name,
            room.electricity_fee,
            room.threshold
        );

        // TODO: 调用实际的通知模块
        // 目前只记录日志，等待通知模块实现
        
        // 模拟通知发送延迟
        sleep(Duration::from_millis(100)).await;

        tracing::info!(
            "通知已发送: room_id={}, roomid={}",
            room.id,
            room.roomid
        );

        Ok(())
    }

    /// 手动触发通知检查（供API调用）
    /// 
    /// # 返回
    /// 处理的房间数量
    pub async fn trigger_manual_check(&self) -> Result<usize> {
        tracing::info!("手动触发通知检查");

        let rooms = self.repository.find_rooms_with_send_flag_true().await?;
        let count = rooms.len();

        for room in rooms {
            if let Err(e) = self.send_notification(&room).await {
                tracing::error!(
                    "发送通知失败: room_id={}, error={}",
                    room.id,
                    e
                );
            }
        }

        Ok(count)
    }
}

/// 通知发送器trait（预留接口）
/// 
/// # 说明
/// 外部模块需要实现此trait以提供通知发送功能
#[allow(dead_code, async_fn_in_trait)]
pub trait NotificationSender: Send + Sync {
    /// 发送电费超限通知
    /// 
    /// # 参数
    /// - `room`: 房间信息
    /// 
    /// # 返回
    /// 是否发送成功
    async fn send_electricity_alert(&self, room: &Room) -> Result<bool>;
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
