-- 回滚迁移：删除触发器、函数、索引和表

-- 删除触发器
DROP TRIGGER IF EXISTS trigger_update_send_flag ON rooms;

-- 删除触发器函数
DROP FUNCTION IF EXISTS update_send_flag();

-- 删除索引
DROP INDEX IF EXISTS idx_rooms_send_flag;
DROP INDEX IF EXISTS idx_rooms_roomid;

-- 删除表
DROP TABLE IF EXISTS rooms;
