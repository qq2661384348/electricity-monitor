//! 邮件模块错误类型

use thiserror::Error;

/// 邮件模块结果类型别名
pub type Result<T> = std::result::Result<T, EmailError>;

/// 邮件发送与模板错误。
#[derive(Debug, Error)]
pub enum EmailError {
    /// 配置缺失或无效
    #[error("邮件配置错误: {0}")]
    Config(String),

    /// 邮箱、验证码或场景校验失败
    #[error("邮件参数校验失败: {0}")]
    Validation(String),

    /// 邮件地址解析失败
    #[error("邮件地址无效: {0}")]
    Address(String),

    /// 邮件消息构建失败
    #[error("邮件消息构建失败: {0}")]
    Build(String),

    /// 邮件模板渲染失败
    #[error("邮件模板错误: {0}")]
    Template(String),

    /// SMTP transport 发送失败
    #[error("邮件发送失败: {message} (重试次数: {retry_count})")]
    Transport { message: String, retry_count: u32 },
}
