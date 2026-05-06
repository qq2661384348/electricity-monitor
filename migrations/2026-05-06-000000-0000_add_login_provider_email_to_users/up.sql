-- 支持 QQ 与邮箱两种登录主体，并保持账号数据按登录渠道隔离。
-- 旧数据全部视为 QQ 账号；邮箱账号使用独立 user.id，绑定与通知状态继续通过 user_id 隔离。
ALTER TABLE users
    ADD COLUMN login_provider VARCHAR(20) NOT NULL DEFAULT 'qq',
    ADD COLUMN email VARCHAR(254);

ALTER TABLE users
    DROP CONSTRAINT IF EXISTS users_qq_number_key;

ALTER TABLE users
    ALTER COLUMN qq_number DROP NOT NULL;

ALTER TABLE users
    ADD CONSTRAINT users_login_provider_check
        CHECK (login_provider IN ('qq', 'email')),
    ADD CONSTRAINT users_identity_shape_check
        CHECK (
            (login_provider = 'qq' AND qq_number IS NOT NULL AND email IS NULL)
            OR
            (login_provider = 'email' AND email IS NOT NULL AND qq_number IS NULL)
        ),
    ADD CONSTRAINT users_login_provider_qq_number_key
        UNIQUE (login_provider, qq_number),
    ADD CONSTRAINT users_login_provider_email_key
        UNIQUE (login_provider, email);

CREATE INDEX idx_users_login_provider ON users(login_provider);
CREATE INDEX idx_users_email ON users(email);
