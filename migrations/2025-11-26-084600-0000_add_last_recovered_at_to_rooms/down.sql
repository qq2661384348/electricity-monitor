-- 回滚：删除 last_recovered_at 字段

ALTER TABLE rooms 
DROP COLUMN IF EXISTS last_recovered_at;
