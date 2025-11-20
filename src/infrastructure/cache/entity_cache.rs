//! 实体缓存层
//! 
//! 提供通用的多级缓存实现，支持：
//! - L1 内存缓存（Moka）
//! - L2 Redis缓存
//! - 缓存穿透保护
//! - 自动失效和更新

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::fmt::Debug;
use std::hash::Hash;

use moka::future::Cache as MokaCache;
use dashmap::DashMap;
use serde::{Serialize, Deserialize};
use deadpool_redis::redis::AsyncCommands;
use tokio::sync::RwLock;
use async_trait::async_trait;

use crate::errors::{Result, AppError};
use crate::infrastructure::RedisPool;

/// 缓存项包装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedItem<T> {
    /// 实际数据
    pub data: T,
    /// 缓存时间
    pub cached_at: i64,
    /// 版本号（用于乐观锁）
    pub version: u32,
}

impl<T> CachedItem<T> {
    /// 创建新的缓存项
    pub fn new(data: T) -> Self {
        Self {
            data,
            cached_at: chrono::Utc::now().timestamp(),
            version: 1,
        }
    }

    /// 检查是否过期
    pub fn is_expired(&self, ttl_seconds: i64) -> bool {
        let now = chrono::Utc::now().timestamp();
        now - self.cached_at > ttl_seconds
    }
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// L1缓存最大容量
    pub l1_max_capacity: u64,
    /// L1缓存TTL（秒）
    pub l1_ttl_seconds: u64,
    /// L1缓存空闲时间（秒）
    pub l1_tti_seconds: u64,
    /// L2缓存TTL（秒）
    pub l2_ttl_seconds: u64,
    /// 是否启用L2缓存
    pub enable_l2: bool,
    /// 是否启用缓存预热
    pub enable_warming: bool,
    /// 缓存穿透保护（空值缓存时间）
    pub null_cache_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_max_capacity: 10_000,
            l1_ttl_seconds: 300,      // 5分钟
            l1_tti_seconds: 60,        // 1分钟空闲
            l2_ttl_seconds: 1800,      // 30分钟
            enable_l2: true,
            enable_warming: true,
            null_cache_seconds: 60,    // 空值缓存1分钟
        }
    }
}

/// 数据加载器trait
#[async_trait]
pub trait DataLoader<K, V>: Send + Sync {
    /// 加载单个数据
    async fn load(&self, key: &K) -> Result<Option<V>>;
    
    /// 批量加载数据
    async fn load_batch(&self, keys: &[K]) -> Result<Vec<(K, V)>>;
}

/// 缓存级别
#[derive(Debug, Clone, Copy)]
pub enum CacheLevel {
    L1,
    L2,
}

/// 缓存统计
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub l1_size: usize,
    pub l1_hit_rate: f64,
    pub l2_hit_rate: f64,
    pub total_hits: u64,
    pub total_misses: u64,
    pub avg_latency_ms: f64,
}

/// 缓存指标
pub struct CacheMetrics {
    l1_hits: std::sync::atomic::AtomicU64,
    l1_misses: std::sync::atomic::AtomicU64,
    l2_hits: std::sync::atomic::AtomicU64,
    l2_misses: std::sync::atomic::AtomicU64,
    total_latency_us: std::sync::atomic::AtomicU64,
    total_requests: std::sync::atomic::AtomicU64,
}

impl CacheMetrics {
    pub fn new() -> Self {
        use std::sync::atomic::AtomicU64;
        Self {
            l1_hits: AtomicU64::new(0),
            l1_misses: AtomicU64::new(0),
            l2_hits: AtomicU64::new(0),
            l2_misses: AtomicU64::new(0),
            total_latency_us: AtomicU64::new(0),
            total_requests: AtomicU64::new(0),
        }
    }

    pub fn record_hit(&self, level: CacheLevel, duration: Duration) {
        use std::sync::atomic::Ordering;
        
        match level {
            CacheLevel::L1 => self.l1_hits.fetch_add(1, Ordering::Relaxed),
            CacheLevel::L2 => self.l2_hits.fetch_add(1, Ordering::Relaxed),
        };
        
        self.total_latency_us.fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self, duration: Duration) {
        use std::sync::atomic::Ordering;
        
        self.l1_misses.fetch_add(1, Ordering::Relaxed);
        self.l2_misses.fetch_add(1, Ordering::Relaxed);
        self.total_latency_us.fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn hit_rate(&self, level: CacheLevel) -> f64 {
        use std::sync::atomic::Ordering;
        
        let (hits, total) = match level {
            CacheLevel::L1 => {
                let h = self.l1_hits.load(Ordering::Relaxed);
                let m = self.l1_misses.load(Ordering::Relaxed);
                (h, h + m)
            }
            CacheLevel::L2 => {
                let h = self.l2_hits.load(Ordering::Relaxed);
                let m = self.l2_misses.load(Ordering::Relaxed);
                (h, h + m)
            }
        };
        
        if total == 0 { 0.0 } else { hits as f64 / total as f64 }
    }

    pub fn total_hits(&self) -> u64 {
        use std::sync::atomic::Ordering;
        self.l1_hits.load(Ordering::Relaxed) + self.l2_hits.load(Ordering::Relaxed)
    }

    pub fn total_misses(&self) -> u64 {
        use std::sync::atomic::Ordering;
        self.l1_misses.load(Ordering::Relaxed) + self.l2_misses.load(Ordering::Relaxed)
    }

    pub fn avg_latency_ms(&self) -> f64 {
        use std::sync::atomic::Ordering;
        
        let total_us = self.total_latency_us.load(Ordering::Relaxed);
        let total_reqs = self.total_requests.load(Ordering::Relaxed);
        
        if total_reqs == 0 { 
            0.0 
        } else { 
            (total_us as f64 / total_reqs as f64) / 1000.0 
        }
    }
}

/// 通用实体缓存
pub struct EntityCache<K, V> 
where
    K: Clone + Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// 缓存键前缀（Redis用）
    prefix: String,
    /// L1内存缓存（Moka）
    l1_cache: MokaCache<K, Arc<Option<CachedItem<V>>>>,
    /// 正在加载的键（防止缓存击穿）
    loading: Arc<DashMap<K, Arc<RwLock<()>>>>,
    /// Redis连接池（L2缓存）
    redis_pool: Option<RedisPool>,
    /// 缓存配置
    config: CacheConfig,
    /// 缓存指标
    metrics: Arc<CacheMetrics>,
}

// 实现部分暂时省略，避免文件过大
