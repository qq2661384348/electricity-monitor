-- 回滚迁移1：删除新增的字段和索引

-- 1. 删除索引
DROP INDEX IF EXISTS idx_rooms_is_active;
DROP INDEX IF EXISTS idx_rooms_primary_hash;
DROP INDEX IF EXISTS idx_rooms_primary_roompath;
DROP INDEX IF EXISTS idx_rooms_roomid;

-- 2. 恢复原来的非唯一索引（如果需要）
CREATE INDEX idx_rooms_roomid ON rooms(roomid);

-- 3. 删除新增字段
ALTER TABLE rooms 
DROP COLUMN IF EXISTS last_synced_at,
DROP COLUMN IF EXISTS external_id,
DROP COLUMN IF EXISTS source_type,
DROP COLUMN IF EXISTS is_active,
DROP COLUMN IF EXISTS has_additional_paths,
DROP COLUMN IF EXISTS primary_roompath_hash,
DROP COLUMN IF EXISTS primary_roompath;
