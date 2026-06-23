-- 新电费接口的 RoomID 可能超过 INT4 范围，房间业务 ID 全链路升为 BIGINT。

ALTER TABLE room_paths
    DROP CONSTRAINT IF EXISTS fk_room_paths_roomid;

ALTER TABLE electricity_history
    DROP CONSTRAINT IF EXISTS fk_electricity_history_roomid;

ALTER TABLE user_room_bindings
    DROP CONSTRAINT IF EXISTS fk_bindings_roomid;

ALTER TABLE rooms
    ALTER COLUMN roomid TYPE BIGINT USING roomid::BIGINT;

ALTER TABLE room_paths
    ALTER COLUMN roomid TYPE BIGINT USING roomid::BIGINT;

ALTER TABLE electricity_history
    ALTER COLUMN roomid TYPE BIGINT USING roomid::BIGINT;

ALTER TABLE user_room_bindings
    ALTER COLUMN roomid TYPE BIGINT USING roomid::BIGINT;

ALTER TABLE room_paths
    ADD CONSTRAINT fk_room_paths_roomid
    FOREIGN KEY (roomid) REFERENCES rooms(roomid)
    ON DELETE CASCADE;

ALTER TABLE electricity_history
    ADD CONSTRAINT fk_electricity_history_roomid
    FOREIGN KEY (roomid) REFERENCES rooms(roomid)
    ON DELETE CASCADE;

ALTER TABLE user_room_bindings
    ADD CONSTRAINT fk_bindings_roomid
    FOREIGN KEY (roomid) REFERENCES rooms(roomid)
    ON DELETE CASCADE;
