//! 多级缓存架构模块
//!
//! 提供高性能的缓存系统，包含：
//! - 基于Moka的内存缓存
//! - TTL和TTI自动过期
//! - 批量操作优化

pub mod cache_manager;
pub mod electricity_loader;
pub mod entity_cache_impl;
pub mod metrics;
pub mod simple_cache;

// 重新导出entity_cache实现
pub use entity_cache_impl as entity_cache;

pub use cache_manager::{CacheManager, CacheManagerConfig, CacheManagerStats};
pub use metrics::CacheMetrics;
pub use simple_cache::{
    BindingCache, RoomCache, SimpleCacheConfig, SimpleCacheManager, SimpleCacheStats, UserCache,
};
