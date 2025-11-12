//! 电费获取基础设施模块
//!
//! 提供批量电费获取功能，包括：
//! - HTTP客户端
//! - 数据解析器
//! - 批量获取执行器

pub mod error;
pub mod fetcher;
pub mod http_client;
pub mod parser;

// 导出常用类型
pub use error::{ElectricityFetchError, Result};
pub use fetcher::{RoomBatchFetcher, RoomResult};
pub use http_client::ReqwestAsyncClient;
pub use parser::ElectricityParser;
