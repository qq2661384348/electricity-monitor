-- 创建电费历史记录表
CREATE TABLE electricity_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    roomid INT4 NOT NULL,
    electricity_fee FLOAT4 NOT NULL,
    recorded_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引（优化查询性能）
CREATE INDEX idx_electricity_history_roomid_recorded 
    ON electricity_history(roomid, recorded_at DESC);

CREATE INDEX idx_electricity_history_recorded 
    ON electricity_history(recorded_at DESC);

-- 添加外键约束（级联删除）
ALTER TABLE electricity_history 
    ADD CONSTRAINT fk_electricity_history_roomid 
    FOREIGN KEY (roomid) REFERENCES rooms(roomid) ON DELETE CASCADE;
