//! 数据库配置

use serde::Deserialize;

/// 数据库类型
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    Postgres,
    Mysql,
}

/// 数据库配置
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库类型
    #[serde(rename = "type")]
    pub db_type: DatabaseType,

    /// 数据库主机
    pub host: String,

    /// 数据库端口
    pub port: u16,

    /// 用户名
    pub username: String,

    /// 密码
    pub password: String,

    /// 密码 secret file 路径
    #[serde(default)]
    pub password_file: Option<String>,

    /// 数据库名
    pub database: String,

    /// 最大连接数
    pub max_connections: u32,

    /// 最小空闲连接数
    pub min_connections: u32,

    /// 连接超时（秒）
    pub connection_timeout: u64,
}

impl DatabaseConfig {
    /// 构建数据库连接URL
    pub fn connection_url(&self) -> String {
        match self.db_type {
            DatabaseType::Postgres => {
                format!(
                    "postgres://{}:{}@{}:{}/{}",
                    self.username, self.password, self.host, self.port, self.database
                )
            }
            DatabaseType::Mysql => {
                format!(
                    "mysql://{}:{}@{}:{}/{}",
                    self.username, self.password, self.host, self.port, self.database
                )
            }
        }
    }
}
