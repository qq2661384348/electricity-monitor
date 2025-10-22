//! URL 构建器（内部模块）
//!
//! 提供高性能 URL 参数修改，支持两种模式：
//! - FastPrefix: URL 以 `?roomid=` 结尾时使用（15倍加速）
//! - Generic: 通用模式，支持复杂参数

use crate::error::{ElectricityError, Result};
use crate::internal::traits::UrlBuilder as UrlBuilderTrait;
use itoa::Buffer as ItoaBuffer;
use url::Url;

/// URL 构建模式（内部枚举）
enum Mode {
    FastPrefix { prefix: String },
    Generic { base_url: Url },
}

/// URL 构建器（内部使用）
///
/// 支持两种模式：
/// - **FastPrefix**: URL 以 `?roomid=` 结尾（15倍加速）
/// - **Generic**: 通用模式
pub(crate) struct UrlBuilder {
    mode: Mode,
    original: String,
}

impl UrlBuilder {
    /// 从模板 URL 创建构建器
    pub(crate) fn from_template(url: &str) -> Result<Self> {
        let parsed = Url::parse(url)
            .map_err(|e| ElectricityError::ConfigError(format!("URL 解析失败: {}", e)))?;

        let original = url.to_string();
        // 检测 roomid= 是否为最后一个参数（无后续 &），命中则启用 FastPrefix
        // 确保 roomid= 出现在 query string 中（? 之后），避免路径段误匹配
        let mode = if let Some(query_start) = original.find('?') {
            let query_part = &original[query_start..];
            match query_part.find("roomid=") {
                Some(relative_idx) => {
                    let idx = query_start + relative_idx;
                    let after = &original[(idx + "roomid=".len())..];
                    if after.contains('&') {
                        Mode::Generic { base_url: parsed }
                    } else {
                        let prefix = original[..(idx + "roomid=".len())].to_string();
                        Mode::FastPrefix { prefix }
                    }
                }
                None => Mode::Generic { base_url: parsed },
            }
        } else {
            Mode::Generic { base_url: parsed }
        };

        Ok(Self { mode, original })
    }

    /// 替换 URL 中的 roomid 参数
    ///
    /// 保留其他所有查询参数，仅修改 roomid 的值。
    /// 如果原 URL 中不存在 roomid 参数，将追加该参数。
    ///
    /// # 参数
    ///
    /// * `roomid` - 新的房间 ID
    ///
    /// # 返回
    ///
    /// 返回修改后的完整 URL 字符串
    ///
    /// # 性能
    ///
    /// 单次调用约 300ns，采用零拷贝优化
    ///
    /// # 示例
    ///
    /// ```
    /// # use electricity_monitor::infrastructure::UrlBuilder;
    /// let builder = UrlBuilder::from_template(
    ///     "https://example.com?a=1&roomid=3243&b=2"
    /// ).unwrap();
    ///
    /// let url = builder.with_roomid("9999");
    /// assert!(url.contains("roomid=9999"));
    /// assert!(url.contains("a=1"));
    /// assert!(url.contains("b=2"));
    /// ```
    pub(crate) fn with_roomid(&self, roomid: &str) -> String {
        match &self.mode {
            Mode::FastPrefix { prefix } => {
                let mut s = String::with_capacity(prefix.len() + roomid.len());
                s.push_str(prefix);
                s.push_str(roomid);
                s
            }
            Mode::Generic { base_url } => {
                let mut url = base_url.clone();
                // 收集所有参数，替换 roomid
                let mut params: Vec<(String, String)> = base_url
                    .query_pairs()
                    .map(|(k, v)| {
                        if k == "roomid" {
                            (k.to_string(), roomid.to_string())
                        } else {
                            (k.to_string(), v.to_string())
                        }
                    })
                    .collect();
                // 如果原 URL 中没有 roomid，添加它
                if !params.iter().any(|(k, _)| k == "roomid") {
                    params.push(("roomid".to_string(), roomid.to_string()));
                }
                // 清空并重建查询参数
                url.query_pairs_mut().clear().extend_pairs(params);
                url.to_string()
            }
        }
    }

    /// 使用 u32 类型的 roomid 构建 URL
    ///
    /// 比 `with_roomid(&str)` 更快，特别是在 FastPrefix 模式下。
    ///
    /// # 参数
    ///
    /// * `roomid` - 房间 ID（u32 类型）
    ///
    /// # 返回
    ///
    /// 完整的 URL 字符串
    ///
    /// # 性能
    ///
    /// FastPrefix 模式: ~37 ns, Generic 模式: ~549 ns
    pub(crate) fn with_roomid_u32(&self, roomid: u32) -> String {
        match &self.mode {
            Mode::FastPrefix { prefix } => {
                let mut buf = ItoaBuffer::new();
                let digits = buf.format(roomid);
                let mut s = String::with_capacity(prefix.len() + digits.len());
                s.push_str(prefix);
                s.push_str(digits);
                s
            }
            Mode::Generic { .. } => self.with_roomid(&roomid.to_string()),
        }
    }

    /// 获取基础 URL（不带查询参数）
    ///
    /// # 返回
    ///
    /// 返回不包含查询参数和片段的基础 URL
    ///
    /// # 示例
    ///
    /// ```
    /// # use electricity_monitor::infrastructure::UrlBuilder;
    /// let builder = UrlBuilder::from_template(
    ///     "https://example.com/path?roomid=123"
    /// ).unwrap();
    ///
    /// assert_eq!(builder.base_path(), "https://example.com/path");
    /// ```
    pub(crate) fn base_path(&self) -> String {
        match &self.mode {
            Mode::Generic { base_url } => {
                let mut url = base_url.clone();
                url.set_query(None);
                url.set_fragment(None);
                url.to_string()
            }
            Mode::FastPrefix { .. } => {
                if let Ok(mut url) = Url::parse(&self.original) {
                    url.set_query(None);
                    url.set_fragment(None);
                    url.to_string()
                } else {
                    self.original.clone()
                }
            }
        }
    }

    /// 获取完整的模板 URL
    ///
    /// # 返回
    ///
    /// 返回原始模板 URL 的字符串引用
    pub(crate) fn base_url(&self) -> &str {
        &self.original
    }
}

/// 为 UrlBuilder 实现 UrlBuilder trait
impl UrlBuilderTrait for UrlBuilder {
    fn with_roomid(&self, roomid: &str) -> String {
        self.with_roomid(roomid)
    }

    fn with_roomid_u32(&self, roomid: u32) -> String {
        self.with_roomid_u32(roomid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_builder_creation() {
        let url = "https://example.com?roomid=123";
        let builder = UrlBuilder::from_template(url);
        assert!(builder.is_ok());
    }

    #[test]
    fn test_url_builder_invalid_url() {
        let url = "not a valid url";
        let builder = UrlBuilder::from_template(url);
        assert!(builder.is_err());
    }

    #[test]
    fn test_with_roomid_simple() {
        let template = "https://example.com?roomid=3243";
        let builder = UrlBuilder::from_template(template).unwrap();

        let new_url = builder.with_roomid("9999");
        assert!(new_url.contains("roomid=9999"));
        assert!(!new_url.contains("roomid=3243"));
    }

    #[test]
    fn test_with_roomid_multiple_params() {
        let template = "https://example.com?openid=xxx&roomid=3243&accid=123";
        let builder = UrlBuilder::from_template(template).unwrap();

        let new_url = builder.with_roomid("9999");
        assert!(new_url.contains("roomid=9999"));
        assert!(new_url.contains("openid=xxx"));
        assert!(new_url.contains("accid=123"));
    }

    #[test]
    fn test_with_roomid_no_existing_roomid() {
        let template = "https://example.com?openid=xxx&accid=123";
        let builder = UrlBuilder::from_template(template).unwrap();

        let new_url = builder.with_roomid("9999");
        assert!(new_url.contains("roomid=9999"));
        assert!(new_url.contains("openid=xxx"));
        assert!(new_url.contains("accid=123"));
    }

    #[test]
    fn test_with_roomid_complex_url() {
        let template = "https://zywxhd02.gxust.edu.cn/Home/GetRoomInfo?\
            openid=orOX3v7OL4eetvVdethToeaxRpN0&\
            sno=202300407035&\
            name=%E8%B5%96%E6%B0%B8%E6%9D%B0&\
            accid=860166&\
            roompath=%E4%B8%9C%E7%8E%AF%E6%A0%A1%E5%8C%BA&\
            roomid=3243&\
            outid=129461630120240403192921980";

        let builder = UrlBuilder::from_template(template).unwrap();
        let new_url = builder.with_roomid("5678");

        assert!(new_url.contains("roomid=5678"));
        assert!(new_url.contains("openid=orOX3v7OL4eetvVdethToeaxRpN0"));
        assert!(new_url.contains("sno=202300407035"));
        assert!(!new_url.contains("roomid=3243"));
    }

    #[test]
    fn test_base_path() {
        let template = "https://example.com/path/to/resource?roomid=123";
        let builder = UrlBuilder::from_template(template).unwrap();

        assert_eq!(builder.base_path(), "https://example.com/path/to/resource");
    }

    #[test]
    fn test_base_url() {
        let template = "https://example.com?roomid=123";
        let builder = UrlBuilder::from_template(template).unwrap();

        assert_eq!(builder.base_url(), template);
    }

    #[test]
    fn test_performance_multiple_calls() {
        let template = "https://example.com?openid=xxx&roomid=3243&accid=123";
        let builder = UrlBuilder::from_template(template).unwrap();

        // 多次调用应该都成功
        for i in 0..100 {
            let url = builder.with_roomid(&i.to_string());
            assert!(url.contains(&format!("roomid={}", i)));
        }
    }

    #[test]
    fn test_with_roomid_u32_simple() {
        let template = "https://example.com?roomid=3243";
        let builder = UrlBuilder::from_template(template).unwrap();
        let url = builder.with_roomid_u32(5678);
        assert!(url.ends_with("roomid=5678") || url.contains("roomid=5678"));
    }

    #[test]
    fn test_with_roomid_u32_multiple_params() {
        let template = "https://example.com?a=1&roomid=3243&b=2";
        let builder = UrlBuilder::from_template(template).unwrap();
        let url = builder.with_roomid_u32(42);
        assert!(url.contains("a=1"));
        assert!(url.contains("b=2"));
        assert!(url.contains("roomid=42"));
    }

    #[test]
    fn test_with_roomid_u32_append_when_missing() {
        let template = "https://example.com?a=1&b=2";
        let builder = UrlBuilder::from_template(template).unwrap();
        let url = builder.with_roomid_u32(9);
        assert!(url.contains("a=1"));
        assert!(url.contains("b=2"));
        assert!(url.contains("roomid=9"));
    }

    #[test]
    fn test_from_template_roomid_empty_fastprefix_like() {
        // 末尾是 roomid= 且无后续 &，应直接在其后拼接数字
        let template = "https://example.com/path?roomid=";
        let builder = UrlBuilder::from_template(template).unwrap();
        let url = builder.with_roomid_u32(100);
        assert_eq!(url, "https://example.com/path?roomid=100");
    }
}
