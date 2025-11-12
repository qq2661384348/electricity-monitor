//! 房间数据爬虫模块
//!
//! 从外部API获取房间列表数据，支持1:N映射场景

pub mod client;
pub mod fetcher;
pub mod models;
pub mod parser;

pub use client::RoomClient;
pub use fetcher::RoomFetcher;
pub use models::{MergeStatistics, RoomData};
