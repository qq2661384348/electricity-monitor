# 废弃代码历史记录

## 概述
本文档记录项目中已删除的废弃代码，用于保存架构演进历史和性能优化过程。

## 1. NotificationService N+1查询问题优化

### 删除时间
2025-11-20

### 删除原因
- 存在严重的N+1查询问题
- 已被 `batch_process_room_notifications` 完全替代
- 性能差异：100个用户从101次查询优化到3次查询

### 废弃代码记录

#### send_room_notifications 方法
```rust
/// 发送房间的所有通知
/// 
/// # 废弃说明
/// **此方法已废弃，存在N+1查询问题！**
/// 
/// 请使用 `batch_process_room_notifications` 批量处理逻辑。
/// 
/// # 参数
/// - `room`: 需要通知的房间
/// 
/// # 返回
/// 成功发送的通知数量
/// 
/// # 说明
/// 1. 查询该房间启用通知的所有绑定
/// 2. ❌ 逐个查询每个绑定用户（N+1问题）
/// 3. 并发发送通知
/// 
/// # 性能问题
/// - 如果房间有100个绑定用户，会执行100次数据库查询
/// - 推荐使用批量查询方法，可将查询次数从N+1降至3次
#[deprecated(
    since = "1.1.0",
    note = "存在N+1查询问题，请使用 batch_process_room_notifications 批量处理"
)]
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
            // 查询用户（N+1问题的根源）
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
        .buffer_unordered(5)
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
```

#### send_notification_to_user 方法
```rust
/// 发送通知给单个用户
/// 
/// # 废弃说明
/// 此方法仅供deprecated的send_room_notifications使用
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
```

### 优化后的替代方案
使用 `batch_process_room_notifications` 方法：
1. 批量查询所有房间
2. 批量查询所有绑定关系
3. 批量查询所有用户
4. 在内存中构建关联关系
5. 并发发送通知

### 性能对比
| 场景 | 废弃方法 | 优化方法 |
|-----|---------|---------|
| 100个房间，每个10个用户 | 1001次查询 | 3次查询 |
| 查询时间 | ~5秒 | <100ms |
| 内存使用 | 低（逐个处理） | 中（批量加载） |

## 2. EntityCache L2缓存预留（保留）

### 保留原因
- L2 Redis缓存即将实现
- 配置已支持 enable_l2 标志
- 删除后需要重新设计接口
- 内存开销极小（<100字节/实例）

### 保留的字段
- `prefix: String` - Redis键前缀
- `redis_pool: Option<RedisPool>` - Redis连接池  
- `config: CacheConfig` - 缓存配置
- `loads: u64` - 统计字段（在stats()方法中使用）

## 版本历史
- v1.1.0 (2025-11-20): 删除NotificationService废弃方法，保留EntityCache预留字段
