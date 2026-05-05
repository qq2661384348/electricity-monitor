//! 邮件发送配置

use serde::Deserialize;

const DEFAULT_SMTP_PORT: u16 = 465;
const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
const DEFAULT_MAX_RETRIES: u32 = 3;
const DEFAULT_RETRY_DELAY_SECONDS: u64 = 2;
const SMTP_PASSWORD_PLACEHOLDER_PREFIX: &str = "CHANGE-THIS";

/// SMTP 邮件发送配置。
///
/// 默认值只负责兼容旧运行时配置；真正发送前仍会由邮件模块校验必填字段。
#[derive(Debug, Clone, Deserialize)]
pub struct EmailConfig {
    /// SMTP 服务器地址
    #[serde(default)]
    pub smtp_host: String,

    /// SMTP 服务器端口
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,

    /// SMTP 登录用户，通常也是发件邮箱
    #[serde(default)]
    pub smtp_user: String,

    /// SMTP 授权码 / 密码
    #[serde(default)]
    pub smtp_password: String,

    /// SMTP 授权码 secret file 路径
    #[serde(default)]
    pub smtp_password_file: Option<String>,

    /// 是否使用 implicit TLS；为 false 时使用 STARTTLS。
    #[serde(default = "default_smtp_use_tls")]
    pub smtp_use_tls: bool,

    /// 发件人显示名
    #[serde(default = "default_from_name")]
    pub from_name: String,

    /// SMTP 命令超时（秒）
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,

    /// 发送失败后的最大重试次数
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// 重试间隔（秒）
    #[serde(default = "default_retry_delay_seconds")]
    pub retry_delay_seconds: u64,
}

fn default_smtp_port() -> u16 {
    DEFAULT_SMTP_PORT
}

fn default_smtp_use_tls() -> bool {
    true
}

fn default_from_name() -> String {
    "Electricity Monitor".to_string()
}

fn default_timeout_seconds() -> u64 {
    DEFAULT_TIMEOUT_SECONDS
}

fn default_max_retries() -> u32 {
    DEFAULT_MAX_RETRIES
}

fn default_retry_delay_seconds() -> u64 {
    DEFAULT_RETRY_DELAY_SECONDS
}

impl EmailConfig {
    /// production 只有在显式配置了 SMTP 发送信息时，才要求 secret file。
    ///
    /// 这样可以让旧部署在尚未接入邮件业务时继续启动；一旦启用邮件发送，
    /// SMTP 授权码仍必须走 secret file，不能退回 tracked 配置原文。
    pub fn requires_secret_file_in_production(&self) -> bool {
        !self.smtp_host.trim().is_empty()
            || !self.smtp_user.trim().is_empty()
            || !self.smtp_password.trim().is_empty()
            || self
                .smtp_password_file
                .as_deref()
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
    }

    pub fn has_valid_resolved_password(&self) -> bool {
        let password = self.smtp_password.trim();
        !password.is_empty() && !password.starts_with(SMTP_PASSWORD_PLACEHOLDER_PREFIX)
    }
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: String::new(),
            smtp_port: default_smtp_port(),
            smtp_user: String::new(),
            smtp_password: String::new(),
            smtp_password_file: None,
            smtp_use_tls: default_smtp_use_tls(),
            from_name: default_from_name(),
            timeout_seconds: default_timeout_seconds(),
            max_retries: default_max_retries(),
            retry_delay_seconds: default_retry_delay_seconds(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_keeps_email_unconfigured() {
        let config = EmailConfig::default();
        assert!(config.smtp_host.is_empty());
        assert!(config.smtp_user.is_empty());
        assert_eq!(config.smtp_port, 465);
        assert!(config.smtp_use_tls);
        assert!(!config.requires_secret_file_in_production());
    }

    #[test]
    fn test_configured_email_requires_secret_file_in_production() {
        let config = EmailConfig {
            smtp_host: "smtp.qq.com".to_string(),
            smtp_user: "cogniaegis@qq.com".to_string(),
            ..EmailConfig::default()
        };

        assert!(config.requires_secret_file_in_production());
    }

    #[test]
    fn test_placeholder_password_is_not_valid_resolved_password() {
        let config = EmailConfig {
            smtp_password: "CHANGE-THIS-EMAIL-SMTP-PASSWORD".to_string(),
            ..EmailConfig::default()
        };

        assert!(!config.has_valid_resolved_password());
    }
}
