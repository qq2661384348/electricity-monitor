//! 通知门控器
//! 
//! 负责管理用户-房间级别的通知发送状态，实现防抖观察期逻辑
//! 
//! # 核心功能
//! 1. 防止向同一用户重复发送同一房间的通知
//! 2. 实现1小时防抖观察期，避免电费抖动导致的重复通知
//! 3. 自动清理已恢复超过观察期的房间状态
//! 4. **持久化支持**: 通知状态持久化到数据库，重启后可恢复
//! 
//! # 设计理念
//! - 内存缓存 + 数据库持久化的混合模式
//! - 启动时从数据库加载历史状态到内存
//! - 写入时双写（内存 + 数据库）
//! - 使用 RwLock 实现读写分离
//! - 独立组件，可插拔设计

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc};

use crate::domain::models::Room;
use crate::errors::Result;
use crate::infrastructure::repositories::{RoomRepository, UserRoomBindingRepository};

/// 通知门控器
/// 
/// # 职责
/// 1. 管理用户-房间级别的通知发送状态
/// 2. 实现防抖观察期逻辑
/// 3. 自动清理过期状态
/// 4. 持久化状态到数据库，支持重启恢复
/// 
/// # 数据存储
/// - **内存层**: `HashMap` 快速查询，启动时从数据库加载
/// - **持久层**: 数据库字段 `user_room_bindings.last_notified_at` 和 `rooms.last_recovered_at`
/// 
/// # 示例
/// ```ignore
/// let gate = NotificationGate::new(Some(Duration::from_secs(3600)));
/// 
/// // 启动时加载历史状态
/// gate.load_from_database(&binding_repo, &room_repo).await?;
/// 
/// // 检查是否应该发送通知
/// if gate.should_notify(user_id, &room).await {
///     // 发送通知
///     send_notification(user_id, &room).await?;
///     // 标记已发送（持久化）
///     gate.mark_notified_persistent(user_id, room.roomid, &binding_repo).await?;
/// }
/// ```
pub struct NotificationGate {
    /// (user_id, roomid) -> 最后通知时间
    /// 
    /// 记录每个用户对每个房间的最后一次通知时间
    /// 使用 NaiveDateTime 支持持久化
    notification_history: Arc<RwLock<HashMap<(Uuid, i32), NaiveDateTime>>>,
    
    /// roomid -> 电费恢复时间
    /// 
    /// 记录房间电费恢复到 >= threshold 的时间
    /// 使用 NaiveDateTime 支持持久化
    room_recovery_time: Arc<RwLock<HashMap<i32, NaiveDateTime>>>,
    
    /// 防抖观察期
    /// 
    /// 当房间电费恢复后，需要等待此时长才能重置通知状态
    /// 默认: 3600秒（1小时）
    debounce_period: Duration,
}

impl NotificationGate {
    /// 创建新的通知门控器
    /// 
    /// # 参数
    /// - `debounce_period`: 防抖观察期，默认为1小时（3600秒）
    /// 
    /// # 返回
    /// 新的 NotificationGate 实例
    /// 
    /// # 示例
    /// ```ignore
    /// // 使用默认1小时观察期
    /// let gate = NotificationGate::new(None);
    /// 
    /// // 自定义30分钟观察期
    /// let gate = NotificationGate::new(Some(Duration::from_secs(1800)));
    /// ```
    pub fn new(debounce_period: Option<Duration>) -> Self {
        Self {
            notification_history: Arc::new(RwLock::new(HashMap::new())),
            room_recovery_time: Arc::new(RwLock::new(HashMap::new())),
            debounce_period: debounce_period.unwrap_or(Duration::from_secs(3600)),
        }
    }
    
    /// 判断是否应该发送通知
    /// 
    /// # 参数
    /// - `user_id`: 用户UUID
    /// - `room`: 房间信息
    /// 
    /// # 返回
    /// - `true`: 允许发送通知
    /// - `false`: 应该阻止发送（已发送过且未过观察期）
    /// 
    /// # 逻辑
    /// 1. 检查用户是否已发送过通知
    /// 2. 如果未发送过，返回 true
    /// 3. 如果已发送过，检查房间是否已恢复且过了观察期
    /// 4. 如果已过观察期，返回 true；否则返回 false
    /// 
    /// # 示例
    /// ```ignore
    /// if gate.should_notify(user.id, &room).await {
    ///     println!("允许发送通知");
    /// } else {
    ///     println!("通知被门控阻止");
    /// }
    /// ```
    pub async fn should_notify(&self, user_id: Uuid, room: &Room) -> bool {
        let key = (user_id, room.roomid);
        
        // 1. 检查通知历史
        let history = self.notification_history.read().await;
        if !history.contains_key(&key) {
            // 从未发送过，允许发送
            drop(history);  // 显式释放读锁
            return true;
        }
        
        drop(history);  // 显式释放读锁
        
        // 2. 已发送过，检查是否满足重置条件
        self.check_recovery_reset(room.roomid).await
    }
    
    /// 检查房间是否已恢复且过了观察期
    /// 
    /// # 参数
    /// - `roomid`: 房间业务ID
    /// 
    /// # 返回
    /// - `true`: 已过观察期，允许重新发送
    /// - `false`: 仍在观察期内或未恢复
    /// 
    /// # 逻辑
    /// 1. 查找房间的恢复时间
    /// 2. 如果未记录恢复时间，说明仍在预警状态，返回 false
    /// 3. 如果已记录恢复时间，计算距离现在的时长
    /// 4. 如果时长 >= debounce_period，返回 true；否则返回 false
    async fn check_recovery_reset(&self, roomid: i32) -> bool {
        let recovery_map = self.room_recovery_time.read().await;
        
        match recovery_map.get(&roomid) {
            Some(recovery_time) => {
                let now = Utc::now().naive_utc();
                let elapsed = now.signed_duration_since(*recovery_time);
                elapsed >= chrono::Duration::from_std(self.debounce_period).unwrap_or(chrono::Duration::hours(1))
            }
            None => {
                // 房间未记录恢复时间，说明仍在预警状态或刚恢复未被监控任务捕获
                false
            }
        }
    }
    
    /// 标记已发送通知（仅内存，不持久化）
    /// 
    /// # 参数
    /// - `user_id`: 用户UUID
    /// - `roomid`: 房间业务ID
    /// 
    /// # 说明
    /// 在成功发送通知后调用此方法，记录发送时间
    /// **注意**: 此方法仅更新内存，不持久化到数据库
    /// 如需持久化，请使用 `mark_notified_persistent` 方法
    pub async fn mark_notified(&self, user_id: Uuid, roomid: i32) {
        let now = Utc::now().naive_utc();
        let mut history = self.notification_history.write().await;
        history.insert((user_id, roomid), now);
        
        tracing::debug!(
            user_id = %user_id,
            roomid = roomid,
            "标记通知已发送（仅内存）"
        );
    }
    
    /// 标记已发送通知并持久化到数据库
    /// 
    /// # 参数
    /// - `user_id`: 用户UUID
    /// - `roomid`: 房间业务ID
    /// - `binding_repo`: 绑定仓储（用于持久化）
    /// 
    /// # 返回
    /// 成功时返回 Ok(())，失败时返回错误
    /// 
    /// # 说明
    /// 双写模式：同时更新内存和数据库，确保重启后状态可恢复
    pub async fn mark_notified_persistent(
        &self,
        user_id: Uuid,
        roomid: i32,
        binding_repo: &UserRoomBindingRepository,
    ) -> Result<()> {
        let now = Utc::now().naive_utc();
        
        // 1. 更新内存
        {
            let mut history = self.notification_history.write().await;
            history.insert((user_id, roomid), now);
        }
        
        // 2. 持久化到数据库
        binding_repo.update_last_notified(user_id, roomid, now).await?;
        
        tracing::debug!(
            user_id = %user_id,
            roomid = roomid,
            time = %now,
            "标记通知已发送（已持久化）"
        );
        
        Ok(())
    }
    
    /// 更新房间恢复状态（仅内存，不持久化）
    /// 
    /// # 参数
    /// - `recovering_rooms`: 电费已恢复（>= threshold）的房间列表
    /// 
    /// # 说明
    /// 由监控任务定期调用，更新房间恢复时间
    /// 仅在首次恢复时记录时间，避免重复更新
    /// **注意**: 此方法仅更新内存，如需持久化请使用 `update_recovery_state_persistent`
    pub async fn update_recovery_state(&self, recovering_rooms: &[Room]) {
        if recovering_rooms.is_empty() {
            return;
        }
        
        let mut recovery_map = self.room_recovery_time.write().await;
        let now = Utc::now().naive_utc();
        let mut new_recoveries = 0;
        
        for room in recovering_rooms {
            // 仅在首次恢复时记录时间
            if recovery_map.insert(room.roomid, now).is_none() {
                new_recoveries += 1;
                
                tracing::debug!(
                    roomid = room.roomid,
                    room_name = &room.room_name,
                    "记录房间恢复时间（仅内存）"
                );
            }
        }
        
        if new_recoveries > 0 {
            tracing::info!(
                new_recoveries = new_recoveries,
                total_recovering = recovery_map.len(),
                "更新房间恢复状态"
            );
        }
    }
    
    /// 更新房间恢复状态并持久化到数据库
    /// 
    /// # 参数
    /// - `recovering_rooms`: 电费已恢复（>= threshold）的房间列表
    /// - `room_repo`: 房间仓储（用于持久化）
    /// 
    /// # 返回
    /// 成功时返回 Ok(())，失败时返回错误
    /// 
    /// # 说明
    /// 双写模式：同时更新内存和数据库，确保重启后状态可恢复
    pub async fn update_recovery_state_persistent(
        &self,
        recovering_rooms: &[Room],
        room_repo: &RoomRepository,
    ) -> Result<()> {
        if recovering_rooms.is_empty() {
            return Ok(());
        }
        
        let now = Utc::now().naive_utc();
        let mut new_recoveries = Vec::new();
        
        // 1. 更新内存并记录新恢复的房间
        {
            let mut recovery_map = self.room_recovery_time.write().await;
            for room in recovering_rooms {
                // 仅在首次恢复时记录时间
                if recovery_map.insert(room.roomid, now).is_none() {
                    new_recoveries.push(room.roomid);
                    
                    tracing::debug!(
                        roomid = room.roomid,
                        room_name = &room.room_name,
                        "记录房间恢复时间"
                    );
                }
            }
        }
        
        // 2. 持久化新恢复的房间到数据库
        for roomid in &new_recoveries {
            room_repo.update_last_recovered(*roomid, now).await?;
        }
        
        if !new_recoveries.is_empty() {
            tracing::info!(
                new_recoveries = new_recoveries.len(),
                "更新房间恢复状态（已持久化）"
            );
        }
        
        Ok(())
    }
    
    /// 清理已恢复超过观察期的房间状态（仅内存）
    /// 
    /// # 返回
    /// 清理的通知记录数量
    /// 
    /// # 工作流程
    /// 1. 找出已过观察期的房间
    /// 2. 清除这些房间的所有用户通知历史
    /// 3. 清除房间恢复时间记录
    /// 
    /// # 说明
    /// 由定期清理任务调用，防止内存泄漏
    /// 清理后，这些房间的通知状态将重置，允许新一轮通知
    pub async fn cleanup_recovered(&self) -> usize {
        let now = Utc::now().naive_utc();
        let debounce_chrono = chrono::Duration::from_std(self.debounce_period)
            .unwrap_or(chrono::Duration::hours(1));
        let mut cleaned_count = 0;
        
        // 1. 找出已过观察期的房间
        let mut recovery_map = self.room_recovery_time.write().await;
        let expired_rooms: Vec<i32> = recovery_map
            .iter()
            .filter(|(_, recovery_time)| {
                now.signed_duration_since(**recovery_time) >= debounce_chrono
            })
            .map(|(roomid, _)| *roomid)
            .collect();
        
        if expired_rooms.is_empty() {
            return 0;
        }
        
        // 2. 清除这些房间的通知历史
        let mut history = self.notification_history.write().await;
        history.retain(|(_, roomid), _| {
            let should_remove = expired_rooms.contains(roomid);
            if should_remove {
                cleaned_count += 1;
            }
            !should_remove
        });
        
        // 3. 清除恢复时间记录
        for roomid in &expired_rooms {
            recovery_map.remove(roomid);
        }
        
        tracing::info!(
            expired_rooms = expired_rooms.len(),
            cleaned_notifications = cleaned_count,
            remaining_recoveries = recovery_map.len(),
            remaining_notifications = history.len(),
            "清理已恢复房间的通知状态"
        );
        
        cleaned_count
    }
    
    /// 清理已恢复超过观察期的房间状态并持久化
    /// 
    /// # 参数
    /// - `binding_repo`: 绑定仓储（用于清理通知历史）
    /// - `room_repo`: 房间仓储（用于清理恢复时间）
    /// 
    /// # 返回
    /// 清理的房间数量
    pub async fn cleanup_recovered_persistent(
        &self,
        binding_repo: &UserRoomBindingRepository,
        room_repo: &RoomRepository,
    ) -> Result<usize> {
        let now = Utc::now().naive_utc();
        let debounce_chrono = chrono::Duration::from_std(self.debounce_period)
            .unwrap_or(chrono::Duration::hours(1));
        
        // 1. 找出已过观察期的房间
        let expired_rooms: Vec<i32>;
        {
            let recovery_map = self.room_recovery_time.read().await;
            expired_rooms = recovery_map
                .iter()
                .filter(|(_, recovery_time)| {
                    now.signed_duration_since(**recovery_time) >= debounce_chrono
                })
                .map(|(roomid, _)| *roomid)
                .collect();
        }
        
        if expired_rooms.is_empty() {
            return Ok(0);
        }
        
        // 2. 清除内存中的通知历史和恢复记录
        {
            let mut recovery_map = self.room_recovery_time.write().await;
            let mut history = self.notification_history.write().await;
            
            history.retain(|(_, roomid), _| !expired_rooms.contains(roomid));
            
            for roomid in &expired_rooms {
                recovery_map.remove(roomid);
            }
        }
        
        // 3. 持久化清理到数据库
        for roomid in &expired_rooms {
            // 重置所有用户的通知时间
            binding_repo.reset_last_notified_by_roomid(*roomid).await?;
            // 重置房间恢复时间
            room_repo.reset_last_recovered(*roomid).await?;
        }
        
        tracing::info!(
            expired_rooms = expired_rooms.len(),
            "清理已恢复房间的通知状态（已持久化）"
        );
        
        Ok(expired_rooms.len())
    }
    
    /// 获取当前状态统计（用于监控和调试）
    /// 
    /// # 返回
    /// (通知历史记录数, 恢复中的房间数)
    pub async fn stats(&self) -> (usize, usize) {
        let history = self.notification_history.read().await;
        let recovery = self.room_recovery_time.read().await;
        
        (history.len(), recovery.len())
    }
    
    // ==================== 持久化加载方法 ====================
    
    /// 从数据库加载历史状态
    /// 
    /// # 参数
    /// - `binding_repo`: 绑定仓储（用于加载通知历史）
    /// - `room_repo`: 房间仓储（用于加载恢复时间）
    /// 
    /// # 返回
    /// 成功时返回 Ok(())，失败时返回错误
    /// 
    /// # 说明
    /// 服务器启动时调用此方法，从数据库恢复历史状态到内存
    /// 这确保了重启后通知状态不会丢失
    pub async fn load_from_database(
        &self,
        binding_repo: &UserRoomBindingRepository,
        room_repo: &RoomRepository,
    ) -> Result<()> {
        // 1. 加载通知历史
        let notification_records = binding_repo.find_all_with_notification_history().await?;
        let notification_count = notification_records.len();
        
        {
            let mut history = self.notification_history.write().await;
            for (user_id, roomid, time) in notification_records {
                history.insert((user_id, roomid), time);
            }
        }
        
        // 2. 加载房间恢复时间
        let recovery_records = room_repo.find_all_with_recovery_time().await?;
        let recovery_count = recovery_records.len();
        
        {
            let mut recovery_map = self.room_recovery_time.write().await;
            for (roomid, time) in recovery_records {
                recovery_map.insert(roomid, time);
            }
        }
        
        tracing::info!(
            notification_history = notification_count,
            recovery_rooms = recovery_count,
            "从数据库加载通知门控状态完成"
        );
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::Room;
    
    /// 创建测试用房间
    fn create_test_room(roomid: i32, electricity_fee: f32, threshold: f32) -> Room {
        Room {
            id: Uuid::new_v4(),
            roomid,
            electricity_fee,
            send_flag: electricity_fee < threshold,
            threshold,
            room_name: format!("测试房间{}", roomid),
            source_type: "manual".to_string(),
            primary_roompath: format!("测试/楼栋/{}", roomid),
            primary_roompath_hash: 0,  // 测试用，实际值不重要
            has_additional_paths: false,
            is_active: true,
            external_id: None,
            last_synced_at: None,
            last_recovered_at: None,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
    
    #[tokio::test]
    async fn test_first_notification_allowed() {
        // 首次通知应该允许
        let gate = NotificationGate::new(None);
        let user_id = Uuid::new_v4();
        let room = create_test_room(101, 50.0, 100.0);
        
        assert!(gate.should_notify(user_id, &room).await);
    }
    
    #[tokio::test]
    async fn test_duplicate_notification_blocked() {
        // 重复通知应该被阻止
        let gate = NotificationGate::new(None);
        let user_id = Uuid::new_v4();
        let room = create_test_room(101, 50.0, 100.0);
        
        // 首次允许
        assert!(gate.should_notify(user_id, &room).await);
        
        // 标记已发送
        gate.mark_notified(user_id, room.roomid).await;
        
        // 再次检查应该被阻止
        assert!(!gate.should_notify(user_id, &room).await);
    }
    
    #[tokio::test]
    async fn test_notification_reset_after_recovery() {
        // 恢复后应该重置状态（但需要过观察期）
        let gate = NotificationGate::new(Some(Duration::from_millis(100)));  // 100ms观察期用于测试
        let user_id = Uuid::new_v4();
        let room = create_test_room(101, 50.0, 100.0);
        
        // 首次发送
        assert!(gate.should_notify(user_id, &room).await);
        gate.mark_notified(user_id, room.roomid).await;
        
        // 重复发送被阻止
        assert!(!gate.should_notify(user_id, &room).await);
        
        // 模拟电费恢复
        let recovered_room = create_test_room(101, 120.0, 100.0);
        gate.update_recovery_state(&[recovered_room.clone()]).await;
        
        // 立即检查，仍在观察期内，应该被阻止
        assert!(!gate.should_notify(user_id, &recovered_room).await);
        
        // 等待观察期结束
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // 观察期结束后，应该允许发送
        assert!(gate.should_notify(user_id, &recovered_room).await);
    }
    
    #[tokio::test]
    async fn test_cleanup_expired_rooms() {
        // 清理任务应该正确清除过期状态
        let gate = NotificationGate::new(Some(Duration::from_millis(100)));
        let user_id = Uuid::new_v4();
        let room = create_test_room(101, 50.0, 100.0);
        
        // 发送通知
        gate.mark_notified(user_id, room.roomid).await;
        
        // 记录恢复时间
        let recovered_room = create_test_room(101, 120.0, 100.0);
        gate.update_recovery_state(&[recovered_room]).await;
        
        // 立即清理，应该没有过期的
        let cleaned = gate.cleanup_recovered().await;
        assert_eq!(cleaned, 0);
        
        // 等待观察期结束
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // 再次清理，应该清除1条记录
        let cleaned = gate.cleanup_recovered().await;
        assert_eq!(cleaned, 1);
        
        // 验证状态已清除
        let (history_count, recovery_count) = gate.stats().await;
        assert_eq!(history_count, 0);
        assert_eq!(recovery_count, 0);
    }
    
    #[tokio::test]
    async fn test_multiple_users_same_room() {
        // 同一房间的多个用户应该独立管理
        let gate = NotificationGate::new(None);
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let room = create_test_room(101, 50.0, 100.0);
        
        // 用户1首次允许
        assert!(gate.should_notify(user1, &room).await);
        gate.mark_notified(user1, room.roomid).await;
        
        // 用户1再次被阻止
        assert!(!gate.should_notify(user1, &room).await);
        
        // 用户2仍然允许（独立状态）
        assert!(gate.should_notify(user2, &room).await);
    }
}

/// 启动房间恢复状态监控任务（带持久化）
/// 
/// # 职责
/// 1. 定期查询电费已恢复（>= threshold）的房间
/// 2. 更新 NotificationGate 的恢复时间记录（持久化到数据库）
/// 3. 定期清理已过观察期的状态（持久化清理）
/// 
/// # 参数
/// - `room_repo`: 房间仓储（用于查询恢复状态和持久化）
/// - `binding_repo`: 绑定仓储（用于持久化通知历史清理）
/// - `gate`: 通知门控器
/// - `interval_secs`: 监控间隔（秒），默认300秒（5分钟）
/// 
/// # 返回
/// Tokio 任务句柄，可用于取消任务
/// 
/// # 工作流程
/// 1. 每隔 interval_secs 秒执行一次
/// 2. 查询所有电费 >= threshold 的房间
/// 3. 调用 gate.update_recovery_state_persistent() 更新恢复状态并持久化
/// 4. 调用 gate.cleanup_recovered_persistent() 清理过期状态并持久化
/// 
/// # 错误处理
/// - 查询失败会记录错误日志，但不会导致任务退出
/// - 任务会在下一个周期自动恢复
/// 
/// # 示例
/// ```ignore
/// use std::sync::Arc;
/// use tokio::time::Duration;
/// 
/// let room_repo = RoomRepository::new(db_pool.clone());
/// let binding_repo = UserRoomBindingRepository::new(db_pool);
/// let gate = Arc::new(NotificationGate::new(None));
/// 
/// // 启动监控任务（每5分钟，带持久化）
/// let handle = spawn_recovery_monitor_persistent(room_repo, binding_repo, gate, 300);
/// 
/// // 需要停止时
/// handle.abort();
/// ```
pub fn spawn_recovery_monitor_persistent(
    room_repo: RoomRepository,
    binding_repo: UserRoomBindingRepository,
    gate: Arc<NotificationGate>,
    interval_secs: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        
        tracing::info!(
            interval_secs = interval_secs,
            "房间恢复监控任务已启动（带持久化）"
        );
        
        loop {
            interval.tick().await;
            
            // 1. 查询已恢复的房间（电费 >= 阈值）
            match room_repo.find_rooms_recovering().await {
                Ok(recovering_rooms) => {
                    if !recovering_rooms.is_empty() {
                        tracing::debug!(
                            count = recovering_rooms.len(),
                            "检测到恢复中的房间"
                        );
                        
                        // 2. 更新恢复状态（持久化）
                        if let Err(e) = gate.update_recovery_state_persistent(&recovering_rooms, &room_repo).await {
                            tracing::error!(
                                "更新房间恢复状态失败: {}",
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "查询恢复中的房间失败: {}",
                        e
                    );
                    // 继续执行，不中断任务
                }
            }
            
            // 3. 清理已过观察期的状态（持久化）
            match gate.cleanup_recovered_persistent(&binding_repo, &room_repo).await {
                Ok(cleaned) if cleaned > 0 => {
                    tracing::info!(
                        cleaned = cleaned,
                        "清理了已恢复房间的通知状态（已持久化）"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "清理过期状态失败: {}",
                        e
                    );
                }
                _ => {}
            }
            
            // 4. 记录状态统计（用于监控）
            let (history_count, recovery_count) = gate.stats().await;
            tracing::debug!(
                notification_history = history_count,
                recovering_rooms = recovery_count,
                "通知门控状态统计"
            );
        }
    })
}

/// 启动房间恢复状态监控任务（仅内存，兼容旧版本）
/// 
/// # 说明
/// 此版本不持久化状态，仅用于兼容或测试场景
/// 生产环境建议使用 `spawn_recovery_monitor_persistent`
pub fn spawn_recovery_monitor(
    room_repo: RoomRepository,
    gate: Arc<NotificationGate>,
    interval_secs: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        
        tracing::info!(
            interval_secs = interval_secs,
            "房间恢复监控任务已启动（仅内存）"
        );
        
        loop {
            interval.tick().await;
            
            // 1. 查询已恢复的房间（电费 >= 阈值）
            match room_repo.find_rooms_recovering().await {
                Ok(recovering_rooms) => {
                    if !recovering_rooms.is_empty() {
                        tracing::debug!(
                            count = recovering_rooms.len(),
                            "检测到恢复中的房间"
                        );
                        
                        // 2. 更新恢复状态（仅内存）
                        gate.update_recovery_state(&recovering_rooms).await;
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "查询恢复中的房间失败: {}",
                        e
                    );
                }
            }
            
            // 3. 清理已过观察期的状态（仅内存）
            let cleaned = gate.cleanup_recovered().await;
            if cleaned > 0 {
                tracing::info!(
                    cleaned = cleaned,
                    "清理了已恢复房间的通知状态"
                );
            }
            
            // 4. 记录状态统计
            let (history_count, recovery_count) = gate.stats().await;
            tracing::debug!(
                notification_history = history_count,
                recovering_rooms = recovery_count,
                "通知门控状态统计"
            );
        }
    })
}
