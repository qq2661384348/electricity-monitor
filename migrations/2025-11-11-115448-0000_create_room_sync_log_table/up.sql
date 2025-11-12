-- 迁移3：创建room_sync_log表（同步日志记录）

CREATE TABLE room_sync_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sync_type VARCHAR(50) NOT NULL,
    started_at TIMESTAMP NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP,
    status VARCHAR(50) NOT NULL,
    stats JSONB,
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX idx_sync_log_started_at ON room_sync_log(started_at DESC);
CREATE INDEX idx_sync_log_status ON room_sync_log(status);
