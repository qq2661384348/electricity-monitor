-- 迁移2：创建room_paths表（1:N扩展表）

CREATE TABLE room_paths (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    roomid BIGINT NOT NULL,
    
    roompath VARCHAR(255) NOT NULL UNIQUE,
    roompath_hash BIGINT NOT NULL,
    room_name VARCHAR(64) NOT NULL,
    
    source_type VARCHAR(50) NOT NULL DEFAULT 'api_sync',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    CONSTRAINT fk_room_paths_roomid 
        FOREIGN KEY (roomid) REFERENCES rooms(roomid) 
        ON DELETE CASCADE
);

-- 创建索引
CREATE UNIQUE INDEX idx_room_paths_roompath ON room_paths(roompath);
CREATE INDEX idx_room_paths_hash ON room_paths(roompath_hash);
CREATE INDEX idx_room_paths_roomid ON room_paths(roomid);

-- 注意：has_additional_paths由应用层维护，不使用触发器
-- 理由：
-- 1. 提高可测试性（可在单元测试中验证）
-- 2. 逻辑透明（同步服务中计算更清晰）
-- 3. 便于调试（避免隐式行为）
-- 实现位置：src/domain/services/room_sync/sync_service.rs
