-- 回滚：删除 last_notified_at 字段

ALTER TABLE user_room_bindings 
DROP COLUMN IF EXISTS last_notified_at;
