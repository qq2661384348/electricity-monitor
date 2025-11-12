-- 回滚迁移3：删除room_sync_log表

DROP TABLE IF EXISTS room_sync_log CASCADE;
