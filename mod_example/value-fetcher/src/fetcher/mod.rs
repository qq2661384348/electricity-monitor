//! 电费获取模块
//!
//! 提供简洁的 Facade API 用于批量查询房间电费

pub mod facade;
pub mod result;

// 公开 API
pub use facade::ElectricityFetcher;
pub use result::FetchResult;
