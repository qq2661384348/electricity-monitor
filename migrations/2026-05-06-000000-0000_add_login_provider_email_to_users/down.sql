-- 回滚到 QQ-only schema 前必须删除邮箱账号及其绑定，避免 qq_number 恢复 NOT NULL 失败。
DELETE FROM user_room_bindings
USING users
WHERE user_room_bindings.user_id = users.id
  AND users.login_provider = 'email';

DELETE FROM users
WHERE login_provider = 'email';

DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_users_login_provider;

ALTER TABLE users
    DROP CONSTRAINT IF EXISTS users_login_provider_email_key,
    DROP CONSTRAINT IF EXISTS users_login_provider_qq_number_key,
    DROP CONSTRAINT IF EXISTS users_identity_shape_check,
    DROP CONSTRAINT IF EXISTS users_login_provider_check;

ALTER TABLE users
    ALTER COLUMN qq_number SET NOT NULL;

ALTER TABLE users
    ADD CONSTRAINT users_qq_number_key UNIQUE (qq_number);

ALTER TABLE users
    DROP COLUMN email,
    DROP COLUMN login_provider;
