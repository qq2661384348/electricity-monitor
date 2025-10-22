//! 配置文件加载器
//!
//! 负责从 INI 文件读取应用程序配置

use crate::error::{ElectricityError, Result};
use configparser::ini::Ini;
use std::collections::HashMap;
use std::path::Path;

/// 配置加载器
///
/// 从 INI 格式的配置文件中加载配置信息
///
/// # 配置文件格式
///
/// ```ini
/// [electric_charge]
/// url = https://example.com/api
///
/// [messager]
/// sender_email = example@email.com
/// ```
///
/// # 示例
///
/// ```no_run
/// use electricity_monitor::config::ConfigLoader;
///
/// let config = ConfigLoader::from_file("config.ini")?;
/// let url = config.get("electric_charge", "url")
///     .ok_or("URL not found")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ConfigLoader {
    config: Ini,
    all_configs: HashMap<String, HashMap<String, Option<String>>>,
}

impl ConfigLoader {
    /// 从文件路径加载配置
    ///
    /// # 参数
    ///
    /// * `path` - 配置文件路径
    ///
    /// # 返回
    ///
    /// 返回 `Result<Self>`，加载失败时返回错误
    ///
    /// # 错误
    ///
    /// - 文件不存在
    /// - 文件格式错误
    /// - 读取权限不足
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use electricity_monitor::config::ConfigLoader;
    /// let config = ConfigLoader::from_file("config.ini")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut config = Ini::new();

        // 加载配置文件
        let all_configs = config
            .load(path.as_ref())
            .map_err(|e| ElectricityError::ConfigError(format!("配置文件加载失败: {}", e)))?;

        Ok(Self {
            config,
            all_configs,
        })
    }

    /// 从字符串内容加载配置
    ///
    /// 主要用于测试场景，允许从内存中的 INI 内容创建配置
    ///
    /// # 参数
    ///
    /// * `content` - INI 格式的配置内容
    ///
    /// # 返回
    ///
    /// 返回 `Result<Self>`，解析失败时返回错误
    ///
    /// # 示例
    ///
    /// ```
    /// # use electricity_monitor::config::ConfigLoader;
    /// let ini_content = r#"
    /// [section]
    /// key = value
    /// "#;
    /// let config = ConfigLoader::from_str(ini_content)?;
    /// assert_eq!(config.get("section", "key"), Some("value".to_string()));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_str(content: &str) -> Result<Self> {
        let mut config = Ini::new();

        // 从字符串读取配置
        let all_configs = config
            .read(content.to_string())
            .map_err(|e| ElectricityError::ConfigError(format!("配置内容解析失败: {}", e)))?;

        Ok(Self {
            config,
            all_configs,
        })
    }

    /// 获取指定配置项的值
    ///
    /// # 参数
    ///
    /// * `section` - 配置节名称（不区分大小写）
    /// * `key` - 配置键名称（不区分大小写）
    ///
    /// # 返回
    ///
    /// - `Some(String)` - 配置值存在
    /// - `None` - 配置值不存在
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use electricity_monitor::config::ConfigLoader;
    /// # let config = ConfigLoader::from_file("config.ini")?;
    /// if let Some(url) = config.get("electric_charge", "url") {
    ///     println!("URL: {}", url);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn get(&self, section: &str, key: &str) -> Option<String> {
        self.config.get(section, key)
    }

    /// 获取指定配置节的所有键值对
    ///
    /// # 参数
    ///
    /// * `section` - 配置节名称
    ///
    /// # 返回
    ///
    /// 返回该节的所有配置项映射，如果节不存在则返回 `None`
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use electricity_monitor::config::ConfigLoader;
    /// # let config = ConfigLoader::from_file("config.ini")?;
    /// if let Some(section) = config.get_section("electric_charge") {
    ///     for (key, value) in section {
    ///         println!("{} = {:?}", key, value);
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn get_section(&self, section: &str) -> Option<&HashMap<String, Option<String>>> {
        self.all_configs.get(section)
    }

    /// 获取所有配置的引用
    ///
    /// # 返回
    ///
    /// 返回包含所有配置节的映射
    pub fn all_configs(&self) -> &HashMap<String, HashMap<String, Option<String>>> {
        &self.all_configs
    }

    /// 检查配置节是否存在
    ///
    /// # 参数
    ///
    /// * `section` - 配置节名称
    ///
    /// # 返回
    ///
    /// 如果配置节存在返回 `true`，否则返回 `false`
    pub fn has_section(&self, section: &str) -> bool {
        self.all_configs.contains_key(section)
    }

    /// 检查配置项是否存在
    ///
    /// # 参数
    ///
    /// * `section` - 配置节名称
    /// * `key` - 配置键名称
    ///
    /// # 返回
    ///
    /// 如果配置项存在返回 `true`，否则返回 `false`
    pub fn has_key(&self, section: &str, key: &str) -> bool {
        self.get(section, key).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_load_config() {
        // 创建临时配置文件
        let temp_path = "test_config.ini";
        let mut file = fs::File::create(temp_path).unwrap();
        writeln!(file, "[test_section]").unwrap();
        writeln!(file, "test_key = test_value").unwrap();

        let config = ConfigLoader::from_file(temp_path);
        assert!(config.is_ok());

        // 清理
        fs::remove_file(temp_path).unwrap();
    }

    #[test]
    fn test_get_value() {
        let temp_path = "test_config2.ini";
        let mut file = fs::File::create(temp_path).unwrap();
        writeln!(file, "[section1]").unwrap();
        writeln!(file, "key1 = value1").unwrap();

        let config = ConfigLoader::from_file(temp_path).unwrap();
        let value = config.get("section1", "key1");
        assert_eq!(value, Some("value1".to_string()));

        fs::remove_file(temp_path).unwrap();
    }
}

// 注：不再实现外部 trait，ConfigLoader 保持独立
