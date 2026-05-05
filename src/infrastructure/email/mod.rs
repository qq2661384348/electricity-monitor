//! 邮件基础设施模块
//!
//! 提供 async SMTP 邮件发送能力，包括普通邮件、HTML 邮件和验证码邮件模板。

pub mod error;
pub mod sender;
pub mod templates;

pub use error::{EmailError, Result};
pub use sender::EmailSender;
pub use templates::{render_verification_code, RenderedEmail, VerificationScene};
