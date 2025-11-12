-- 回滚：删除电费历史记录表（级联删除索引和约束）
DROP TABLE IF EXISTS electricity_history;
