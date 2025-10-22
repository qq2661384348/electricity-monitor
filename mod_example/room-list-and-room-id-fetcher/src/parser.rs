//! JSON 解析器
//!
//! 处理特殊的 JSON 格式：
//! - UTF-8 BOM 头（\uFEFF）
//! - 双重 JSON 编码（JSON 字符串中嵌套 JSON）

use anyhow::{Context, Result};
use serde_json::Value;

/// 安全解析 JSON（处理 BOM + 双重编码）
///
/// # 参数
/// - `text`: 原始响应文本
///
/// # 返回
/// - `Ok(Value)`: 解析成功的 JSON 值
/// - `Err`: 解析失败的错误信息
///
/// # 处理流程
/// 1. 去除 UTF-8 BOM 头（`\u{FEFF}`）
/// 2. 第一次 JSON 解析
/// 3. 如果结果是字符串，进行第二次解析（双重编码）
///
/// # 示例
/// ```rust
/// use room_fetcher::parser::safe_parse;
///
/// // 正常 JSON
/// let result = safe_parse(r#"{"key": "value"}"#).unwrap();
///
/// // 带 BOM 的 JSON
/// let result = safe_parse("\u{FEFF}{\"key\": \"value\"}").unwrap();
///
/// // 双重编码的 JSON
/// let result = safe_parse(r#""{\"key\": \"value\"}""#).unwrap();
/// ```
#[inline]
pub fn safe_parse(text: &str) -> Result<Value> {
    tracing::debug!("开始解析 JSON，原始长度: {} 字节", text.len());

    // 1. 去除 BOM 头
    let clean = text.strip_prefix('\u{FEFF}').unwrap_or(text);

    if clean != text {
        tracing::debug!("检测到并移除 UTF-8 BOM 头");
    }

    // 2. 第一次解析
    let mut value: Value = serde_json::from_str(clean).with_context(|| {
        let preview = if clean.len() > 500 {
            format!("{}...", &clean[..500])
        } else {
            clean.to_string()
        };
        format!("第一次 JSON 解析失败，内容: {}", preview)
    })?;

    // 3. 检测双重编码（如果是字符串，再解析一次）
    if let Value::String(inner) = value {
        tracing::debug!("检测到双重 JSON 编码，进行第二次解析");

        value = serde_json::from_str(&inner).with_context(|| {
            let preview = if inner.len() > 500 {
                format!("{}...", &inner[..500])
            } else {
                inner.clone()
            };
            format!("第二次 JSON 解析失败（双重编码），内容: {}", preview)
        })?;
    }

    tracing::debug!("JSON 解析成功");
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_json() {
        let json = r#"{"component": [{"RoomDepId": "123", "DepName": "测试"}]}"#;
        let result = safe_parse(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_with_bom() {
        let json = "\u{FEFF}{\"component\": []}";
        let result = safe_parse(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_double_encoded_json() {
        let json = r#""{\"component\": []}""#;
        let result = safe_parse(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let json = "invalid json {";
        let result = safe_parse(json);
        assert!(result.is_err());
    }
}
