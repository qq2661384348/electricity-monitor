-- 回滚到费用型逻辑

-- 删除余额型触发器
DROP TRIGGER IF EXISTS trigger_update_send_flag ON rooms;
DROP FUNCTION IF EXISTS update_send_flag();

-- 恢复费用型触发器函数（原逻辑）
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

-- 创建触发器
CREATE TRIGGER trigger_update_send_flag
BEFORE INSERT OR UPDATE OF electricity_fee, threshold ON rooms
FOR EACH ROW
EXECUTE FUNCTION update_send_flag();

-- 重新计算所有房间的send_flag（恢复原逻辑）
UPDATE rooms SET electricity_fee = electricity_fee;
