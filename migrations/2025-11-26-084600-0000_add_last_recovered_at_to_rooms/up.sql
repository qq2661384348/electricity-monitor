-- 为 rooms 表添加 last_recovered_at 字段
-- 用于持久化房间电费恢复时间，防止服务器重启后防抖逻辑失效

ALTER TABLE rooms 
ADD COLUMN last_recovered_at TIMESTAMP NULL;

-- 添加注释说明字段用途
COMMENT ON COLUMN rooms.last_recovered_at IS '房间电费恢复到阈值以上的时间，用于防抖观察期计算';
