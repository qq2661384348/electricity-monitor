//! 公开站点访问配置

use serde::Deserialize;

/// 公开站点访问配置
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PublicSiteConfig {
    /// 对外访问域名，不包含协议和路径
    pub domain: String,

    /// 对外访问端口。使用字符串是为了让配置模板可以保持空值，并在运行时做显式校验。
    pub port: String,
}

impl PublicSiteConfig {
    /// 生成通知消息中展示给用户的公开访问地址。
    pub fn public_url(&self) -> String {
        format!("https://{}:{}/", self.domain.trim(), self.port.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_keeps_template_values_empty() {
        let config = PublicSiteConfig::default();

        assert!(config.domain.is_empty());
        assert!(config.port.is_empty());
    }

    #[test]
    fn public_url_uses_configured_domain_and_port() {
        let config = PublicSiteConfig {
            domain: " pythonrust.icu ".to_string(),
            port: " 11451 ".to_string(),
        };

        assert_eq!(config.public_url(), "https://pythonrust.icu:11451/");
    }
}
