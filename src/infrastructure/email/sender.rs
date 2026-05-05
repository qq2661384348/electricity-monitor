//! SMTP 邮件发送器

use std::sync::LazyLock;
use std::time::Duration;

use lettre::message::{header::ContentType, Mailbox, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use regex::Regex;
use tokio::time::sleep;
use tracing::instrument;

use crate::config::EmailConfig;

use super::error::{EmailError, Result};
use super::templates::{render_verification_code, VerificationScene};

static EMAIL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$")
        .expect("email regex must compile")
});

/// Async SMTP 邮件发送器。
#[derive(Clone)]
pub struct EmailSender {
    config: EmailConfig,
}

impl EmailSender {
    /// 创建邮件发送器。
    ///
    /// 构造阶段只校验配置，不创建 SMTP transport，也不会连接 SMTP 服务器。
    ///
    /// `lettre` 的 async transport 内部持有 Tokio 驱动的连接池；把 transport 延迟到
    /// 发送时创建，可以避免在同步初始化或单元测试上下文中析构连接池时要求 reactor。
    pub fn new(config: EmailConfig) -> Result<Self> {
        Self::validate_config(&config)?;
        Ok(Self { config })
    }

    /// 发送普通邮件。
    #[instrument(skip(self, body), fields(external_dependency = "email_smtp", to_email = %to_email))]
    pub async fn send_email(
        &self,
        to_email: &str,
        subject: &str,
        body: &str,
        is_html: bool,
    ) -> Result<()> {
        let message = if is_html {
            self.build_alternative_message(to_email, subject, &strip_html_tags(body), body)?
        } else {
            self.build_text_message(to_email, subject, body)?
        };

        self.send_with_retry(message, to_email).await
    }

    /// 发送 HTML 邮件。
    pub async fn send_html_email(&self, to_email: &str, subject: &str, html: &str) -> Result<()> {
        self.send_email(to_email, subject, html, true).await
    }

    /// 发送验证码邮件。
    pub async fn send_verification_code(
        &self,
        to_email: &str,
        code: &str,
        scene: &str,
    ) -> Result<()> {
        let scene = VerificationScene::try_from(scene)?;
        let rendered = render_verification_code(code, scene, &self.config.from_name)?;
        let message = self.build_alternative_message(
            to_email,
            &rendered.subject,
            &rendered.text_body,
            &rendered.html_body,
        )?;

        self.send_with_retry(message, to_email).await
    }

    /// 构建纯文本邮件消息。公开给测试和后续业务集成做轻量校验。
    pub fn build_text_message(&self, to_email: &str, subject: &str, body: &str) -> Result<Message> {
        let from = self.sender_mailbox()?;
        let to = parse_recipient(to_email)?;

        Message::builder()
            .from(from)
            .to(to)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|error| EmailError::Build(error.to_string()))
    }

    fn build_alternative_message(
        &self,
        to_email: &str,
        subject: &str,
        text_body: &str,
        html_body: &str,
    ) -> Result<Message> {
        let from = self.sender_mailbox()?;
        let to = parse_recipient(to_email)?;

        Message::builder()
            .from(from)
            .to(to)
            .subject(subject)
            .multipart(MultiPart::alternative_plain_html(
                text_body.to_string(),
                html_body.to_string(),
            ))
            .map_err(|error| EmailError::Build(error.to_string()))
    }

    async fn send_with_retry(&self, message: Message, to_email: &str) -> Result<()> {
        let mailer = self.build_mailer()?;

        for attempt in 0..=self.config.max_retries {
            match mailer.send(message.clone()).await {
                Ok(_) => {
                    tracing::info!(to_email = to_email, "邮件发送成功");
                    return Ok(());
                }
                Err(error) if attempt < self.config.max_retries => {
                    tracing::warn!(
                        to_email = to_email,
                        attempt = attempt + 1,
                        max_retries = self.config.max_retries,
                        error = %error,
                        "邮件发送失败，准备重试"
                    );

                    if self.config.retry_delay_seconds > 0 {
                        sleep(Duration::from_secs(self.config.retry_delay_seconds)).await;
                    }
                }
                Err(error) => {
                    return Err(EmailError::Transport {
                        message: error.to_string(),
                        retry_count: attempt,
                    });
                }
            }
        }

        Err(EmailError::Transport {
            message: "SMTP 发送未返回结果".to_string(),
            retry_count: self.config.max_retries,
        })
    }

    fn sender_mailbox(&self) -> Result<Mailbox> {
        let address = parse_email_address(&self.config.smtp_user)?;
        let display_name = self.config.from_name.trim();
        let name = if display_name.is_empty() {
            None
        } else {
            Some(display_name.to_string())
        };

        Ok(Mailbox::new(name, address))
    }

    fn build_mailer(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        let credentials = Credentials::new(
            self.config.smtp_user.clone(),
            self.config.smtp_password.clone(),
        );
        let builder = if self.config.smtp_use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.smtp_host)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.config.smtp_host)
        }
        .map_err(|error| EmailError::Config(error.to_string()))?;

        Ok(builder
            .credentials(credentials)
            .port(self.config.smtp_port)
            .timeout(Some(Duration::from_secs(self.config.timeout_seconds)))
            .build())
    }

    fn validate_config(config: &EmailConfig) -> Result<()> {
        let missing = [
            ("email.smtp_host", config.smtp_host.trim()),
            ("email.smtp_user", config.smtp_user.trim()),
            ("email.smtp_password", config.smtp_password.trim()),
        ]
        .into_iter()
        .filter_map(|(field, value)| if value.is_empty() { Some(field) } else { None })
        .collect::<Vec<_>>();

        if !missing.is_empty() {
            return Err(EmailError::Config(format!(
                "缺少必填字段: {}",
                missing.join(", ")
            )));
        }

        if config.smtp_port == 0 {
            return Err(EmailError::Config(
                "email.smtp_port 必须是 1 到 65535 之间的端口".to_string(),
            ));
        }

        if config.timeout_seconds == 0 {
            return Err(EmailError::Config(
                "email.timeout_seconds 必须大于 0".to_string(),
            ));
        }

        parse_email_address(&config.smtp_user)
            .map(|_| ())
            .map_err(|error| EmailError::Config(error.to_string()))
    }
}

fn parse_recipient(to_email: &str) -> Result<Mailbox> {
    let address = parse_email_address(to_email)?;
    Ok(Mailbox::new(None, address))
}

fn parse_email_address(email: &str) -> Result<lettre::Address> {
    let trimmed = email.trim();
    if !EMAIL_PATTERN.is_match(trimmed) {
        return Err(EmailError::Address(trimmed.to_string()));
    }

    trimmed
        .parse::<lettre::Address>()
        .map_err(|error| EmailError::Address(error.to_string()))
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> EmailConfig {
        EmailConfig {
            smtp_host: "smtp.qq.com".to_string(),
            smtp_port: 465,
            smtp_user: "cogniaegis@qq.com".to_string(),
            smtp_password: "test-password".to_string(),
            smtp_password_file: None,
            smtp_use_tls: true,
            from_name: "CogniAegis".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_seconds: 2,
        }
    }

    #[test]
    fn test_sender_creation_with_valid_config() {
        let sender = EmailSender::new(test_config());
        assert!(sender.is_ok());
    }

    #[test]
    fn test_sender_rejects_missing_required_config() {
        let config = EmailConfig {
            smtp_host: String::new(),
            ..test_config()
        };
        let error = match EmailSender::new(config) {
            Ok(_) => panic!("缺少 smtp_host 时应拒绝创建 EmailSender"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("email.smtp_host"));
    }

    #[test]
    fn test_sender_rejects_invalid_smtp_user() {
        let config = EmailConfig {
            smtp_user: "invalid-email".to_string(),
            ..test_config()
        };
        let error = match EmailSender::new(config) {
            Ok(_) => panic!("smtp_user 不是邮箱格式时应拒绝创建 EmailSender"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("invalid-email"));
    }

    #[test]
    fn test_build_text_message() {
        let sender = EmailSender::new(test_config()).unwrap();
        let message = sender
            .build_text_message("user@example.com", "测试邮件", "hello")
            .unwrap();
        let formatted = String::from_utf8(message.formatted()).unwrap();

        assert!(formatted.contains("To: user@example.com"));
        assert!(formatted.contains("Subject:"));
    }

    #[test]
    fn test_build_text_message_rejects_invalid_recipient() {
        let sender = EmailSender::new(test_config()).unwrap();
        let error = sender
            .build_text_message("invalid-email", "测试邮件", "hello")
            .unwrap_err();

        assert!(error.to_string().contains("invalid-email"));
    }

    #[test]
    fn test_strip_html_tags_keeps_text() {
        assert_eq!(
            strip_html_tags("<p>Hello <strong>World</strong></p>"),
            "Hello World"
        );
    }
}
