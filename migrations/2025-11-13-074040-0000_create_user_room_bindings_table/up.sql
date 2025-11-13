-- Create user_room_bindings table for N:M relationship
CREATE TABLE user_room_bindings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    roomid INT4 NOT NULL,
    notification_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    -- Unique constraint: one user can only bind to a room once
    UNIQUE(user_id, roomid)
);

-- Create indexes for user_room_bindings table
CREATE INDEX idx_bindings_user_id ON user_room_bindings(user_id);
CREATE INDEX idx_bindings_roomid ON user_room_bindings(roomid);
CREATE INDEX idx_bindings_notification ON user_room_bindings(notification_enabled) 
    WHERE notification_enabled = true;

-- Add foreign key constraint to rooms table
ALTER TABLE user_room_bindings 
ADD CONSTRAINT fk_bindings_roomid 
FOREIGN KEY (roomid) REFERENCES rooms(roomid) ON DELETE CASCADE;
