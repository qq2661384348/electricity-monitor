//! 验证工具
//!
//! 提供常用的验证正则表达式

use once_cell::sync::Lazy;
use regex::Regex;

/// QQ号验证正则表达式（5-20位数字）
pub static QQ_NUMBER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\d{5,20}$").expect("QQ号正则表达式编译失败"));

/// 邮箱地址验证正则表达式。
///
/// 这里只做登录入口的基础格式门禁；真正发送前 `lettre` 仍会再次解析地址。
pub static EMAIL_ADDRESS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,254}$")
        .expect("邮箱正则表达式编译失败")
});

/// 验证码正则表达式（6位数字）
pub static VERIFICATION_CODE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\d{6}$").expect("验证码正则表达式编译失败"));

/// 统一邮箱登录标识：去除首尾空白并转小写，保证同一邮箱不会因大小写重复开户。
pub fn normalize_email_address(input: &str) -> Option<String> {
    let normalized = input.trim().to_ascii_lowercase();
    if EMAIL_ADDRESS_REGEX.is_match(&normalized) {
        Some(normalized)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qq_number_regex() {
        assert!(QQ_NUMBER_REGEX.is_match("12345"));
        assert!(QQ_NUMBER_REGEX.is_match("123456789"));
        assert!(QQ_NUMBER_REGEX.is_match("12345678901234567890"));

        assert!(!QQ_NUMBER_REGEX.is_match("1234")); // 太短
        assert!(!QQ_NUMBER_REGEX.is_match("123456789012345678901")); // 太长
        assert!(!QQ_NUMBER_REGEX.is_match("abc123")); // 包含字母
    }

    #[test]
    fn test_normalize_email_address() {
        assert_eq!(
            normalize_email_address(" Student@Example.COM "),
            Some("student@example.com".to_string())
        );
        assert_eq!(normalize_email_address("invalid-email"), None);
    }

    #[test]
    fn test_verification_code_regex() {
        assert!(VERIFICATION_CODE_REGEX.is_match("123456"));
        assert!(VERIFICATION_CODE_REGEX.is_match("000000"));

        assert!(!VERIFICATION_CODE_REGEX.is_match("12345")); // 太短
        assert!(!VERIFICATION_CODE_REGEX.is_match("1234567")); // 太长
        assert!(!VERIFICATION_CODE_REGEX.is_match("12345a")); // 包含字母
    }
}
