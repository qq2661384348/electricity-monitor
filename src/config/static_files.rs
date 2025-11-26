//! 静态文件服务配置

use serde::Deserialize;

/// 静态文件服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct StaticFilesConfig {
    /// 是否启用静态文件服务
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// 静态文件目录
    #[serde(default = "default_directory")]
    pub directory: String,
    
    /// 入口文件（SPA fallback）
    #[serde(default = "default_index_file")]
    pub index_file: String,
    
    /// 带 hash 资源的缓存时间（秒），默认1年
    #[serde(default = "default_cache_max_age")]
    pub cache_max_age_seconds: u64,
    
    /// HTML 文件缓存时间（秒），默认0表示 no-cache
    #[serde(default)]
    pub html_cache_seconds: u64,
}

fn default_enabled() -> bool {
    true
}

fn default_directory() -> String {
    "static".to_string()
}

fn default_index_file() -> String {
    "index.html".to_string()
}

fn default_cache_max_age() -> u64 {
    31536000 // 1年
}

impl Default for StaticFilesConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            directory: default_directory(),
            index_file: default_index_file(),
            cache_max_age_seconds: default_cache_max_age(),
            html_cache_seconds: 0,
        }
    }
}

impl StaticFilesConfig {
    /// 获取入口文件的完整路径
    pub fn index_path(&self) -> String {
        format!("{}/{}", self.directory, self.index_file)
    }
    
    /// 检查目录是否存在
    pub fn directory_exists(&self) -> bool {
        std::path::Path::new(&self.directory).is_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = StaticFilesConfig::default();
        assert!(config.enabled);
        assert_eq!(config.directory, "static");
        assert_eq!(config.index_file, "index.html");
        assert_eq!(config.cache_max_age_seconds, 31536000);
        assert_eq!(config.html_cache_seconds, 0);
    }

    #[test]
    fn test_index_path() {
        let config = StaticFilesConfig::default();
        assert_eq!(config.index_path(), "static/index.html");
    }
}
