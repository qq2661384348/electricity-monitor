//! 房间同步服务模块
//!
//! 负责从外部系统同步房间数据

pub mod crawler;
pub mod sync_service;

pub use crawler::{MergeStatistics, RoomClient, RoomData, RoomFetcher};
pub use sync_service::{RoomSyncService, SyncStats};
