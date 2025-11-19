-- 修改电费预警逻辑：从"费用超过阈值"改为"余额低于阈值"
-- 业务含义：electricity_fee 是剩余余额，threshold 是预警线

-- 删除旧触发器
DROP TRIGGER IF EXISTS trigger_update_send_flag ON rooms;
DROP FUNCTION IF EXISTS update_send_flag();

-- 创建新触发器函数（余额型逻辑）
CREATE OR REPLACE FUNCTION update_send_flag()
RETURNS TRIGGER AS $$
BEGIN
    -- 如果电费余额低于阈值，设置send_flag为true（预警）
    IF NEW.electricity_fee < NEW.threshold THEN
        NEW.send_flag := TRUE;
    ELSE
        -- 余额充足时重置标志
        NEW.send_flag := FALSE;
    END IF;
    
    -- 更新时间戳
    NEW.updated_at := NOW();
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 创建新触发器
CREATE TRIGGER trigger_update_send_flag
BEFORE INSERT OR UPDATE OF electricity_fee, threshold ON rooms
FOR EACH ROW
EXECUTE FUNCTION update_send_flag();

-- 重新计算所有房间的send_flag（立即生效）
UPDATE rooms SET electricity_fee = electricity_fee;
