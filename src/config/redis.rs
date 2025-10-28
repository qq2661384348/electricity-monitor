//! Redis配置

use serde::Deserialize;

/// Redis配置
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    /// Redis主机
    pub host: String,
    
    /// Redis端口
    pub port: u16,
    
    /// 最大连接数
    pub max_connections: u32,
    
    /// 最小空闲连接数
    pub min_connections: u32,
    
    /// 连接超时（秒）
    pub connection_timeout: u64,
}

impl RedisConfig {
    /// 构建Redis连接URL
    pub fn connection_url(&self) -> String {
        format!("redis://{}:{}", self.host, self.port)
    }
}
