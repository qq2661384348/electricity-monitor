//! 电费获取服务配置

use serde::Deserialize;

/// 电费获取服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct ElectricityFetcherConfig {
    /// 是否启用
    pub enabled: bool,
    /// 电费查询API URL（必须以?roomid=结尾）
    pub api_url: String,
    /// 电费获取间隔（分钟）
    pub fetch_interval_minutes: u64,
    /// 历史记录间隔（小时）
    pub history_interval_hours: u64,
    /// 历史数据保留天数
    pub history_retention_days: i64,
}

impl Default for ElectricityFetcherConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            api_url: "https://example.com/api?roomid=".to_string(),
            fetch_interval_minutes: 5,
            history_interval_hours: 1,
            history_retention_days: 8,
        }
    }
}

impl ElectricityFetcherConfig {
    /// 验证配置有效性
    ///
    /// # 返回
    /// - Ok(()) 配置有效
    /// - Err(String) 配置错误信息
    pub fn validate(&self) -> Result<(), String> {
        // 验证API URL格式
        if !self.api_url.ends_with("?roomid=") {
            return Err(format!(
                "api_url必须以'?roomid='结尾，当前值: {}",
                self.api_url
            ));
        }
        
        // 验证间隔时间
        if self.fetch_interval_minutes == 0 {
            return Err("fetch_interval_minutes必须大于0".to_string());
        }
        
        if self.history_interval_hours == 0 {
            return Err("history_interval_hours必须大于0".to_string());
        }
        
        // 验证保留天数
        if self.history_retention_days < 1 {
            return Err("history_retention_days必须至少为1天".to_string());
        }
        
        Ok(())
    }
}
