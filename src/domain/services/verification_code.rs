//! 验证码服务

use crate::config::VerificationConfig;
use crate::errors::{AppError, Result};
use crate::infrastructure::notification::QQClient;
use crate::infrastructure::RedisPool;
use rand::Rng;
use redis::AsyncCommands;

/// 验证码服务
pub struct VerificationCodeService {
    /// Redis连接池
    redis_pool: RedisPool,
    
    /// QQ客户端
    qq_client: QQClient,
    
    /// 配置
    config: VerificationConfig,
}

impl VerificationCodeService {
    /// 创建验证码服务
    /// 
    /// # 参数
    /// * `redis_pool` - Redis连接池
    /// * `qq_client` - QQ客户端
    /// * `config` - 验证码配置
    pub fn new(redis_pool: RedisPool, qq_client: QQClient, config: VerificationConfig) -> Self {
        Self {
            redis_pool,
            qq_client,
            config,
        }
    }
    
    /// 生成验证码
    /// 
    /// # 返回
    /// 6位数字验证码字符串
    fn generate_code(&self) -> String {
        let mut rng = rand::rng();
        let code: u32 = rng.random_range(100000..1000000);
        code.to_string()
    }
    
    /// 发送验证码并存储
    /// 
    /// 流程：
    /// 1. 生成验证码
    /// 2. 通过QQ机器人发送验证码
    /// 3. 发送成功后存储到Redis（key: "verify:{qq_number}", ttl: 300秒）
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
        
        // 1. 生成验证码
        let code = self.generate_code();
        
        // 2. 通过QQ机器人发送验证码
        // 直接使用 ? 操作符，让 From<NotificationError> trait 自动转换
        // 这样可以保留 UserNotFriend 等特定错误类型，而不是强制转换为 Internal
        self.qq_client
            .send_verification_code(qq_number, &code)
            .await?;
        
        tracing::info!(
            qq_number = qq_number,
            "验证码发送成功，开始存储到Redis"
        );
        
        // 3. 发送成功后存储到Redis
        let key = self.config.redis_key(qq_number);
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
            qq_number = qq_number,
            expire_seconds = self.config.expire_seconds,
            "验证码已存储到Redis"
        );
        
        Ok(code)
    }
    
    /// 验证验证码
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
        tracing::debug!(
            qq_number = qq_number,
            "验证验证码"
        );
        
        let key = self.config.redis_key(qq_number);
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
                    qq_number = qq_number,
                    "验证码验证成功（已自动删除）"
                );
                Ok(true)
            }
            Some(_) => {
                tracing::warn!(
                    qq_number = qq_number,
                    "验证码不匹配"
                );
                Ok(false)
            }
            None => {
                tracing::warn!(
                    qq_number = qq_number,
                    "验证码不存在或已过期"
                );
                Ok(false)
            }
        }
    }
    
    /// 检查验证码是否存在（不删除）
    /// 
    /// # 参数
    /// * `qq_number` - QQ号
    /// 
    /// # 返回
    /// 验证码是否存在
    pub async fn code_exists(&self, qq_number: &str) -> Result<bool> {
        let key = self.config.redis_key(qq_number);
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
        let redis_pool = RedisPool::builder(
            deadpool_redis::Manager::new("redis://127.0.0.1:6379").unwrap()
        )
        .build()
        .unwrap();
        
        let qq_client = QQClient::new(crate::config::QQBotConfig::default()).unwrap();
        let config = VerificationConfig::default();
        let service = VerificationCodeService::new(redis_pool, qq_client, config);
        
        let code = service.generate_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_redis_key_generation() {
        let config = VerificationConfig::default();
        let key = config.redis_key("123456");
        assert_eq!(key, "verify:123456");
    }
}
