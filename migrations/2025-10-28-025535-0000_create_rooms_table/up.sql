-- 创建rooms表
CREATE TABLE rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    roomid BIGINT NOT NULL,
    electricity_fee REAL NOT NULL DEFAULT 0.0,
    send_flag BOOLEAN NOT NULL DEFAULT FALSE,
    threshold REAL NOT NULL,
    room_name VARCHAR(64) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- 创建索引优化查询性能
-- roomid索引：用于通过roomid查找房间
CREATE INDEX idx_rooms_roomid ON rooms(roomid);

-- send_flag部分索引：只索引为true的记录，优化查询效率
CREATE INDEX idx_rooms_send_flag ON rooms(send_flag) WHERE send_flag = TRUE;

-- 创建触发器函数：自动更新send_flag和updated_at
CREATE OR REPLACE FUNCTION update_send_flag()
RETURNS TRIGGER AS $$
BEGIN
    -- 如果电费超过阈值，设置send_flag为true
    IF NEW.electricity_fee > NEW.threshold THEN
        NEW.send_flag := TRUE;
    END IF;
    
    -- 更新时间戳
    NEW.updated_at := NOW();
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 创建触发器：在INSERT或UPDATE时自动执行
CREATE TRIGGER trigger_update_send_flag
BEFORE INSERT OR UPDATE OF electricity_fee, threshold ON rooms
FOR EACH ROW
EXECUTE FUNCTION update_send_flag();
