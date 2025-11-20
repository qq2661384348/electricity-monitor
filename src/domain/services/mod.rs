//! 领域服务
//!
//! 封装跨实体的业务逻辑

pub mod electricity_fetcher_service;
pub mod captcha_verification;
pub mod electricity_service;
pub mod notification_cache;
pub mod notification_gate;
pub mod notification_service;
pub mod rate_limiter;
pub mod room_path_tree;
pub mod room_sync;
pub mod room_sync_cache;
pub mod roomid_cache;
pub mod verification_code;

pub use electricity_fetcher_service::{ElectricityFetcherService, FetchStatistics};
pub use electricity_service::{ElectricityData, ElectricityService};
pub use notification_cache::{CacheStats, NotificationCache};
pub use notification_gate::{spawn_recovery_monitor, NotificationGate};
pub use notification_service::{NotificationService, NotificationStats};
pub use rate_limiter::{RateLimiter, RateLimitOperation};
pub use room_path_tree::{PathChildNode, RoomPathTree};
pub use room_sync::{MergeStatistics, RoomClient, RoomData, RoomFetcher, RoomSyncService, SyncStats};
pub use room_sync_cache::RoomSyncCache;
pub use roomid_cache::RoomIdCache;
pub use verification_code::VerificationCodeService;
