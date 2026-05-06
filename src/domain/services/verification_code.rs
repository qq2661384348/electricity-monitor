//! 验证码服务

use crate::config::VerificationConfig;
use crate::errors::{AppError, Result};
use crate::infrastructure::email::EmailDelivery;
use crate::infrastructure::notification::QQClient;
use crate::infrastructure::RedisPool;
use rand::Rng;
use redis::AsyncCommands;

/// 验证码服务
pub struct VerificationCodeService {
    /// Redis连接池
    redis_pool: RedisPool,

    /// 配置
    config: VerificationConfig,
}

impl VerificationCodeService {
    /// 创建验证码服务
    ///
    /// # 参数
    /// * `redis_pool` - Redis连接池
    /// * `config` - 验证码配置
    pub fn new(redis_pool: RedisPool, config: VerificationConfig) -> Self {
        Self { redis_pool, config }
    }

    /// 生成验证码
    ///
    /// # 返回
    /// 按配置长度生成的数字验证码字符串
    pub fn generate_code(&self) -> String {
        let mut rng = rand::rng();
        (0..self.config.code_length)
            .map(|_| char::from(b'0' + rng.random_range(0..10) as u8))
            .collect()
    }

    /// 通过 QQ 机器人发送验证码并存储。
    ///
    /// 这是旧调用点兼容入口；新登录链路应显式调用 `send_and_store_qq`
    /// 或 `send_and_store_email`，避免未来新增渠道时误用 QQ 默认行为。
    ///
    /// # 参数
    /// * `qq_number` - QQ号
    ///
    /// # 返回
    /// 验证码（仅用于测试和日志）
    ///
    /// # 错误
    /// - QQ机器人发送失败
    /// - Redis存储失败
    pub async fn send_and_store(&self, qq_number: &str) -> Result<String> {
        tracing::info!(qq_number = qq_number, "生成并发送验证码");

        let code = self.generate_code();

        self.send_qq_code(qq_number, &code).await?;
        self.store_code_for("qq", qq_number, &code).await?;

        Ok(code)
    }

    pub async fn send_and_store_qq(&self, qq_client: &QQClient, qq_number: &str) -> Result<String> {
        tracing::info!(qq_number = qq_number, "生成并发送 QQ 验证码");

        let code = self.generate_code();
        qq_client.send_verification_code(qq_number, &code).await?;
        self.store_code_for("qq", qq_number, &code).await?;

        Ok(code)
    }

    pub async fn send_and_store_email(
        &self,
        email_sender: &dyn EmailDelivery,
        email: &str,
    ) -> Result<String> {
        tracing::info!(email = email, "生成并发送邮箱验证码");

        let code = self.generate_code();
        email_sender
            .send_verification_code(email, &code, "login")
            .await?;
        self.store_code_for("email", email, &code).await?;

        Ok(code)
    }

    async fn send_qq_code(&self, qq_number: &str, code: &str) -> Result<()> {
        let qq_client = QQClient::new(crate::config::AppConfig::global().qq_bot.clone())
            .map_err(|e| AppError::Internal(format!("QQ客户端初始化失败: {}", e)))?;
        qq_client.send_verification_code(qq_number, code).await?;
        Ok(())
    }

    pub async fn store_code_for(
        &self,
        login_provider: &str,
        identifier: &str,
        code: &str,
    ) -> Result<()> {
        tracing::info!(
            login_provider = login_provider,
            identifier = identifier,
            "验证码发送成功，开始存储到Redis"
        );

        let key = self.config.redis_key_for(login_provider, identifier);
        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| AppError::Redis(format!("获取Redis连接失败: {}", e)))?;

        // 设置验证码，有效期为配置的过期时间
        conn.set_ex::<_, _, ()>(&key, &code, self.config.expire_seconds)
            .await
            .map_err(|e| AppError::Redis(format!("存储验证码失败: {}", e)))?;

        tracing::info!(
            login_provider = login_provider,
            identifier = identifier,
            expire_seconds = self.config.expire_seconds,
            "验证码已存储到Redis"
        );
        Ok(())
    }

    /// 验证 QQ 登录验证码。
    ///
    /// # 参数
    /// * `qq_number` - QQ号
    /// * `code` - 用户输入的验证码
    ///
    /// # 返回
    /// 验证是否通过
    ///
    /// # 错误
    /// - Redis连接失败
    ///
    /// # 实现说明
    /// 使用`get_del`命令（Redis 6.2+）实现原子的GET+DEL操作，
    /// 确保验证码只能被使用一次，避免并发竞态条件
    pub async fn verify_code(&self, qq_number: &str, code: &str) -> Result<bool> {
        self.verify_code_for("qq", qq_number, code).await
    }

    pub async fn verify_code_for(
        &self,
        login_provider: &str,
        identifier: &str,
        code: &str,
    ) -> Result<bool> {
        tracing::debug!(
            login_provider = login_provider,
            identifier = identifier,
            "验证验证码"
        );

        let key = self.config.redis_key_for(login_provider, identifier);
        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| AppError::Redis(format!("获取Redis连接失败: {}", e)))?;

        // 使用get_del原子操作：获取并删除验证码（Redis 6.2+）
        // 这确保了即使并发请求，验证码也只能被使用一次
        let stored_code: Option<String> = conn
            .get_del(&key)
            .await
            .map_err(|e| AppError::Redis(format!("GETDEL验证码失败: {}", e)))?;

        match stored_code {
            Some(stored) if stored == code => {
                tracing::info!(
                    login_provider = login_provider,
                    identifier = identifier,
                    "验证码验证成功（已自动删除）"
                );
                Ok(true)
            }
            Some(_) => {
                tracing::warn!(
                    login_provider = login_provider,
                    identifier = identifier,
                    "验证码不匹配"
                );
                Ok(false)
            }
            None => {
                tracing::warn!(
                    login_provider = login_provider,
                    identifier = identifier,
                    "验证码不存在或已过期"
                );
                Ok(false)
            }
        }
    }

    /// 检查 QQ 登录验证码是否存在（不删除）。
    ///
    /// # 参数
    /// * `qq_number` - QQ号
    ///
    /// # 返回
    /// 验证码是否存在
    pub async fn code_exists(&self, qq_number: &str) -> Result<bool> {
        self.code_exists_for("qq", qq_number).await
    }

    pub async fn code_exists_for(&self, login_provider: &str, identifier: &str) -> Result<bool> {
        let key = self.config.redis_key_for(login_provider, identifier);
        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| AppError::Redis(format!("获取Redis连接失败: {}", e)))?;

        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| AppError::Redis(format!("检查验证码存在性失败: {}", e)))?;

        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code() {
        let redis_pool =
            RedisPool::builder(deadpool_redis::Manager::new("redis://127.0.0.1:6379").unwrap())
                .build()
                .unwrap();

        let config = VerificationConfig::default();
        let service = VerificationCodeService::new(redis_pool, config);

        let code = service.generate_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_redis_key_generation() {
        let config = VerificationConfig::default();
        let key = config.redis_key("123456");
        assert_eq!(key, "verify:qq:123456");
        assert_eq!(
            config.redis_key_for("email", "student@example.com"),
            "verify:email:student@example.com"
        );
    }
}
