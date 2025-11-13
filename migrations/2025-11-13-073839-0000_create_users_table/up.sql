-- Create users table for authentication
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    qq_number VARCHAR(20) NOT NULL UNIQUE,
    role VARCHAR(20) NOT NULL DEFAULT 'user',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create indexes for users table
CREATE INDEX idx_users_qq_number ON users(qq_number);
CREATE INDEX idx_users_role ON users(role);

-- Insert default admin user
INSERT INTO users (qq_number, role, is_active)
VALUES ('100000001', 'admin', true);
