-- 迁移1：修改rooms表，新增同步相关字段

-- 1. 添加roomid UNIQUE约束（如果不存在）
DROP INDEX IF EXISTS idx_rooms_roomid;
CREATE UNIQUE INDEX idx_rooms_roomid ON rooms(roomid);

-- 2. 新增字段
ALTER TABLE rooms 
ADD COLUMN IF NOT EXISTS primary_roompath VARCHAR(255),
ADD COLUMN IF NOT EXISTS primary_roompath_hash BIGINT,
ADD COLUMN IF NOT EXISTS has_additional_paths BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE,
ADD COLUMN IF NOT EXISTS source_type VARCHAR(50) NOT NULL DEFAULT 'manual',
ADD COLUMN IF NOT EXISTS external_id VARCHAR(100),
ADD COLUMN IF NOT EXISTS last_synced_at TIMESTAMP;

-- 3. 数据迁移：将room_name转为primary_roompath（临时方案）
UPDATE rooms 
SET primary_roompath = COALESCE(room_name, '未知路径/' || roomid::TEXT),
    primary_roompath_hash = 0  -- 临时值，后续同步时更新
WHERE primary_roompath IS NULL;

-- 4. 设置非空约束
ALTER TABLE rooms 
ALTER COLUMN primary_roompath SET NOT NULL,
ALTER COLUMN primary_roompath_hash SET NOT NULL;

-- 5. 创建索引
CREATE UNIQUE INDEX idx_rooms_primary_roompath ON rooms(primary_roompath);
CREATE INDEX idx_rooms_primary_hash ON rooms(primary_roompath_hash);
CREATE INDEX idx_rooms_is_active ON rooms(is_active) WHERE is_active = TRUE;
