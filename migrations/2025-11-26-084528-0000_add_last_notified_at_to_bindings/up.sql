-- 为 user_room_bindings 表添加 last_notified_at 字段
-- 用于持久化通知历史，防止服务器重启后重复发送通知

ALTER TABLE user_room_bindings 
ADD COLUMN last_notified_at TIMESTAMP NULL;

-- 添加注释说明字段用途
COMMENT ON COLUMN user_room_bindings.last_notified_at IS '最后一次发送通知的时间，用于防止重复通知';
