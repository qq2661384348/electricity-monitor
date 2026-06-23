//! EntityCache实现
//!
//! 提供通用的多级缓存实现

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use dashmap::DashMap;
use moka::future::Cache as MokaCache;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::metrics::CacheMetrics;
use crate::errors::Result;
use crate::infrastructure::RedisPool;

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

/// 缓存项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedItem<V> {
    /// 实际数据
    pub data: V,
    /// 缓存时间戳
    pub cached_at: u64,
    /// 版本号（用于缓存失效）
    pub version: u32,
}

/// 数据加载器trait
#[async_trait]
pub trait DataLoader<K, V>: Send + Sync {
    /// 加载单个数据
    async fn load(&self, key: &K) -> Result<Option<V>>;

    /// 批量加载数据
    async fn load_batch(&self, keys: &[K]) -> Result<Vec<(K, V)>>
    where
        K: Clone;
}

/// 缓存统计
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// L1命中次数
    pub l1_hits: u64,
    /// L1未命中次数
    pub l1_misses: u64,
    /// L2命中次数
    pub l2_hits: u64,
    /// L2未命中次数
    pub l2_misses: u64,
    /// 加载次数
    #[allow(dead_code)]
    pub loads: u64,
}

/// 通用实体缓存
pub struct EntityCache<K, V>
where
    K: Clone + Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// 缓存键前缀（Redis用）
    #[allow(dead_code)]
    prefix: String,
    /// L1内存缓存（Moka）
    l1_cache: MokaCache<K, Arc<Option<CachedItem<V>>>>,
    /// 正在加载的键（防止缓存击穿）
    loading: Arc<DashMap<K, Arc<RwLock<()>>>>,
    /// Redis连接池（L2缓存）
    #[allow(dead_code)]
    redis_pool: Option<RedisPool>,
    /// 缓存配置
    #[allow(dead_code)]
    config: CacheConfig,
    /// 缓存指标
    metrics: Arc<CacheMetrics>,
}

impl<K, V> EntityCache<K, V>
where
    K: Clone + Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// 创建新的缓存实例
    pub fn new(
        prefix: impl Into<String>,
        config: CacheConfig,
        redis_pool: Option<RedisPool>,
    ) -> Self {
        let l1_cache = MokaCache::builder()
            .max_capacity(config.l1_max_capacity)
            .time_to_live(Duration::from_secs(config.l1_ttl_seconds))
            .time_to_idle(Duration::from_secs(config.l1_tti_seconds))
            .build();

        Self {
            prefix: prefix.into(),
            l1_cache,
            loading: Arc::new(DashMap::new()),
            redis_pool,
            config,
            metrics: Arc::new(CacheMetrics::new()),
        }
    }

    /// 获取或加载数据
    pub async fn get_or_load<L>(&self, key: K, loader: &L) -> Result<Option<V>>
    where
        L: DataLoader<K, V>,
    {
        let start = std::time::Instant::now();

        // 1. 检查L1缓存
        if let Some(cached) = self.l1_cache.get(&key).await {
            self.metrics
                .record_hit(super::metrics::CacheLevel::L1, start.elapsed());
            if let Some(item) = cached.as_ref() {
                return Ok(Some(item.data.clone()));
            }
            return Ok(None); // 缓存了空值
        }

        self.metrics
            .record_miss(super::metrics::CacheLevel::L1, start.elapsed());

        // 2. 防止缓存击穿
        let lock = self
            .loading
            .entry(key.clone())
            .or_insert_with(|| Arc::new(RwLock::new(())))
            .clone();

        let _guard = lock.write().await;

        // 3. 双重检查
        if let Some(cached) = self.l1_cache.get(&key).await {
            if let Some(item) = cached.as_ref() {
                return Ok(Some(item.data.clone()));
            }
            return Ok(None);
        }

        // 4. 从加载器加载
        let value = loader.load(&key).await?;

        // 5. 缓存结果
        let cached_item = value.as_ref().map(|v| CachedItem {
            data: v.clone(),
            cached_at: current_timestamp(),
            version: 1,
        });

        self.l1_cache
            .insert(key.clone(), Arc::new(cached_item.clone()))
            .await;

        // 清理loading状态
        self.loading.remove(&key);

        Ok(value)
    }

    /// 批量获取数据
    pub async fn get_batch<L>(&self, keys: Vec<K>, loader: &L) -> Result<Vec<(K, Option<V>)>>
    where
        L: DataLoader<K, V>,
    {
        let mut results = Vec::with_capacity(keys.len());
        let mut missing_keys = Vec::new();

        // 1. 检查缓存
        for key in &keys {
            let start = std::time::Instant::now();
            if let Some(cached) = self.l1_cache.get(key).await {
                self.metrics
                    .record_hit(super::metrics::CacheLevel::L1, start.elapsed());
                let value = cached.as_ref().as_ref().map(|item| item.data.clone());
                results.push((key.clone(), value));
            } else {
                self.metrics
                    .record_miss(super::metrics::CacheLevel::L1, start.elapsed());
                missing_keys.push(key.clone());
            }
        }

        // 2. 批量加载缺失的
        if !missing_keys.is_empty() {
            let loaded = loader.load_batch(&missing_keys).await?;

            // 转换为HashMap便于查找
            let loaded_map: HashMap<K, V> = loaded.into_iter().collect();

            for key in missing_keys {
                let value = loaded_map.get(&key).cloned();

                // 缓存结果
                let cached_item = value.as_ref().map(|v| CachedItem {
                    data: v.clone(),
                    cached_at: current_timestamp(),
                    version: 1,
                });

                self.l1_cache
                    .insert(key.clone(), Arc::new(cached_item))
                    .await;
                results.push((key, value));
            }
        }

        Ok(results)
    }

    /// 设置缓存值
    pub async fn set(&self, key: K, value: V) -> Result<()> {
        let cached_item = CachedItem {
            data: value,
            cached_at: current_timestamp(),
            version: 1,
        };

        self.l1_cache.insert(key, Arc::new(Some(cached_item))).await;
        Ok(())
    }

    /// 使缓存失效
    pub async fn invalidate(&self, key: &K) -> Result<()> {
        self.l1_cache.invalidate(key).await;
        Ok(())
    }

    /// 使所有缓存失效
    pub async fn invalidate_all(&self) -> Result<()> {
        self.l1_cache.invalidate_all();
        Ok(())
    }

    /// 获取缓存统计
    pub fn stats(&self) -> CacheStats {
        let report = self.metrics.report();
        CacheStats {
            l1_hits: report.l1_hits,
            l1_misses: report.l1_misses,
            l2_hits: report.l2_hits,
            l2_misses: report.l2_misses,
            loads: report.total_requests,
        }
    }
}

/// 获取当前时间戳（秒）
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestLoader;

    #[async_trait]
    impl DataLoader<i64, String> for TestLoader {
        async fn load(&self, key: &i64) -> Result<Option<String>> {
            Ok(Some(format!("value_{}", key)))
        }

        async fn load_batch(&self, keys: &[i64]) -> Result<Vec<(i64, String)>> {
            Ok(keys.iter().map(|k| (*k, format!("value_{}", k))).collect())
        }
    }

    #[tokio::test]
    async fn test_cache_basic() {
        let config = CacheConfig {
            l1_max_capacity: 100,
            l1_ttl_seconds: 60,
            l1_tti_seconds: 30,
            l2_ttl_seconds: 300,
            enable_l2: false,
            enable_warming: false,
            null_cache_seconds: 0,
        };

        let cache = EntityCache::<i64, String>::new("test", config, None);
        let loader = TestLoader;

        // 第一次加载
        let value = cache.get_or_load(1, &loader).await.unwrap();
        assert_eq!(value, Some("value_1".to_string()));

        // 第二次从缓存获取
        let value = cache.get_or_load(1, &loader).await.unwrap();
        assert_eq!(value, Some("value_1".to_string()));

        // 检查统计
        let stats = cache.stats();
        assert_eq!(stats.l1_hits, 1); // 第二次请求命中
        assert_eq!(stats.l1_misses, 1); // 第一次请求未命中
        assert_eq!(stats.loads, 2); // 总请求数 = hits + misses
    }
}
