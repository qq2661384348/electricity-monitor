//! 缓存指标监控模块
//! 
//! 提供详细的缓存性能指标和监控

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// 缓存操作类型
#[derive(Debug, Clone, Copy)]
pub enum CacheOperation {
    Get,
    Set,
    Delete,
    BatchGet,
    BatchSet,
    BatchDelete,
}

/// 缓存级别
#[derive(Debug, Clone, Copy)]
pub enum CacheLevel {
    L1,
    L2,
}

/// 缓存指标
pub struct CacheMetrics {
    // L1缓存指标
    l1_hits: AtomicU64,
    l1_misses: AtomicU64,
    l1_sets: AtomicU64,
    l1_deletes: AtomicU64,
    l1_total_latency_us: AtomicU64,
    
    // L2缓存指标
    l2_hits: AtomicU64,
    l2_misses: AtomicU64,
    l2_sets: AtomicU64,
    l2_deletes: AtomicU64,
    l2_total_latency_us: AtomicU64,
    
    // 总体指标
    total_requests: AtomicU64,
    cache_stampede_prevented: AtomicU64,
    null_cache_hits: AtomicU64,
    
    // 时间戳
    started_at: Instant,
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self {
            l1_hits: AtomicU64::new(0),
            l1_misses: AtomicU64::new(0),
            l1_sets: AtomicU64::new(0),
            l1_deletes: AtomicU64::new(0),
            l1_total_latency_us: AtomicU64::new(0),
            
            l2_hits: AtomicU64::new(0),
            l2_misses: AtomicU64::new(0),
            l2_sets: AtomicU64::new(0),
            l2_deletes: AtomicU64::new(0),
            l2_total_latency_us: AtomicU64::new(0),
            
            total_requests: AtomicU64::new(0),
            cache_stampede_prevented: AtomicU64::new(0),
            null_cache_hits: AtomicU64::new(0),
            
            started_at: Instant::now(),
        }
    }
}

impl CacheMetrics {
    /// 创建新的指标实例
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 记录缓存命中
    pub fn record_hit(&self, level: CacheLevel, duration: Duration) {
        match level {
            CacheLevel::L1 => {
                self.l1_hits.fetch_add(1, Ordering::Relaxed);
                self.l1_total_latency_us
                    .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
            }
            CacheLevel::L2 => {
                self.l2_hits.fetch_add(1, Ordering::Relaxed);
                self.l2_total_latency_us
                    .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
            }
        }
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录缓存未中
    pub fn record_miss(&self, level: CacheLevel, duration: Duration) {
        match level {
            CacheLevel::L1 => {
                self.l1_misses.fetch_add(1, Ordering::Relaxed);
                self.l1_total_latency_us
                    .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
            }
            CacheLevel::L2 => {
                self.l2_misses.fetch_add(1, Ordering::Relaxed);
                self.l2_total_latency_us
                    .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
            }
        }
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录缓存设置
    pub fn record_set(&self, level: CacheLevel, duration: Duration) {
        match level {
            CacheLevel::L1 => {
                self.l1_sets.fetch_add(1, Ordering::Relaxed);
                self.l1_total_latency_us
                    .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
            }
            CacheLevel::L2 => {
                self.l2_sets.fetch_add(1, Ordering::Relaxed);
                self.l2_total_latency_us
                    .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
            }
        }
    }
    
    /// 记录缓存删除
    pub fn record_delete(&self, level: CacheLevel, duration: Duration) {
        match level {
            CacheLevel::L1 => {
                self.l1_deletes.fetch_add(1, Ordering::Relaxed);
                self.l1_total_latency_us
                    .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
            }
            CacheLevel::L2 => {
                self.l2_deletes.fetch_add(1, Ordering::Relaxed);
                self.l2_total_latency_us
                    .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
            }
        }
    }
    
    /// 记录防止缓存击穿
    pub fn record_stampede_prevented(&self) {
        self.cache_stampede_prevented.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录空值缓存命中
    pub fn record_null_cache_hit(&self) {
        self.null_cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 获取L1命中率
    pub fn l1_hit_rate(&self) -> f64 {
        let hits = self.l1_hits.load(Ordering::Relaxed);
        let misses = self.l1_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
    
    /// 获取L2命中率
    pub fn l2_hit_rate(&self) -> f64 {
        let hits = self.l2_hits.load(Ordering::Relaxed);
        let misses = self.l2_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
    
    /// 获取总体命中率
    pub fn overall_hit_rate(&self) -> f64 {
        let l1_hits = self.l1_hits.load(Ordering::Relaxed);
        let l2_hits = self.l2_hits.load(Ordering::Relaxed);
        let total_hits = l1_hits + l2_hits;
        
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        
        if total_requests == 0 {
            0.0
        } else {
            total_hits as f64 / total_requests as f64
        }
    }
    
    /// 获取L1平均延迟（微秒）
    pub fn l1_avg_latency_us(&self) -> f64 {
        let total_latency = self.l1_total_latency_us.load(Ordering::Relaxed);
        let total_ops = self.l1_hits.load(Ordering::Relaxed)
            + self.l1_misses.load(Ordering::Relaxed)
            + self.l1_sets.load(Ordering::Relaxed)
            + self.l1_deletes.load(Ordering::Relaxed);
        
        if total_ops == 0 {
            0.0
        } else {
            total_latency as f64 / total_ops as f64
        }
    }
    
    /// 获取L2平均延迟（微秒）
    pub fn l2_avg_latency_us(&self) -> f64 {
        let total_latency = self.l2_total_latency_us.load(Ordering::Relaxed);
        let total_ops = self.l2_hits.load(Ordering::Relaxed)
            + self.l2_misses.load(Ordering::Relaxed)
            + self.l2_sets.load(Ordering::Relaxed)
            + self.l2_deletes.load(Ordering::Relaxed);
        
        if total_ops == 0 {
            0.0
        } else {
            total_latency as f64 / total_ops as f64
        }
    }
    
    /// 获取防止缓存击穿次数
    pub fn stampede_prevented_count(&self) -> u64 {
        self.cache_stampede_prevented.load(Ordering::Relaxed)
    }
    
    /// 获取空值缓存命中次数
    pub fn null_cache_hit_count(&self) -> u64 {
        self.null_cache_hits.load(Ordering::Relaxed)
    }
    
    /// 获取运行时长（秒）
    pub fn uptime_seconds(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }
    
    /// 获取详细报告
    pub fn report(&self) -> CacheMetricsReport {
        CacheMetricsReport {
            l1_hits: self.l1_hits.load(Ordering::Relaxed),
            l1_misses: self.l1_misses.load(Ordering::Relaxed),
            l1_hit_rate: self.l1_hit_rate(),
            l1_avg_latency_us: self.l1_avg_latency_us(),
            
            l2_hits: self.l2_hits.load(Ordering::Relaxed),
            l2_misses: self.l2_misses.load(Ordering::Relaxed),
            l2_hit_rate: self.l2_hit_rate(),
            l2_avg_latency_us: self.l2_avg_latency_us(),
            
            overall_hit_rate: self.overall_hit_rate(),
            total_requests: self.total_requests.load(Ordering::Relaxed),
            stampede_prevented: self.stampede_prevented_count(),
            null_cache_hits: self.null_cache_hit_count(),
            uptime_seconds: self.uptime_seconds(),
        }
    }
    
    /// 重置指标
    pub fn reset(&self) {
        self.l1_hits.store(0, Ordering::Relaxed);
        self.l1_misses.store(0, Ordering::Relaxed);
        self.l1_sets.store(0, Ordering::Relaxed);
        self.l1_deletes.store(0, Ordering::Relaxed);
        self.l1_total_latency_us.store(0, Ordering::Relaxed);
        
        self.l2_hits.store(0, Ordering::Relaxed);
        self.l2_misses.store(0, Ordering::Relaxed);
        self.l2_sets.store(0, Ordering::Relaxed);
        self.l2_deletes.store(0, Ordering::Relaxed);
        self.l2_total_latency_us.store(0, Ordering::Relaxed);
        
        self.total_requests.store(0, Ordering::Relaxed);
        self.cache_stampede_prevented.store(0, Ordering::Relaxed);
        self.null_cache_hits.store(0, Ordering::Relaxed);
    }
}

/// 缓存指标报告
#[derive(Debug, Clone)]
pub struct CacheMetricsReport {
    // L1指标
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l1_hit_rate: f64,
    pub l1_avg_latency_us: f64,
    
    // L2指标
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub l2_hit_rate: f64,
    pub l2_avg_latency_us: f64,
    
    // 总体指标
    pub overall_hit_rate: f64,
    pub total_requests: u64,
    pub stampede_prevented: u64,
    pub null_cache_hits: u64,
    pub uptime_seconds: u64,
}

impl CacheMetricsReport {
    /// 打印报告
    pub fn log_report(&self) {
        tracing::info!("=== 缓存性能报告 ===");
        tracing::info!(
            "L1缓存: 命中={} 未中={} 命中率={:.2}% 平均延迟={:.2}μs",
            self.l1_hits,
            self.l1_misses,
            self.l1_hit_rate * 100.0,
            self.l1_avg_latency_us
        );
        tracing::info!(
            "L2缓存: 命中={} 未中={} 命中率={:.2}% 平均延迟={:.2}μs",
            self.l2_hits,
            self.l2_misses,
            self.l2_hit_rate * 100.0,
            self.l2_avg_latency_us
        );
        tracing::info!(
            "总体: 命中率={:.2}% 总请求={} 防止击穿={} 空值命中={}",
            self.overall_hit_rate * 100.0,
            self.total_requests,
            self.stampede_prevented,
            self.null_cache_hits
        );
        tracing::info!("运行时长: {}秒", self.uptime_seconds);
        
        // 性能优化建议
        if self.l1_hit_rate < 0.7 {
            tracing::warn!("L1命中率偏低，建议增加L1缓存容量或调整TTL");
        }
        
        if self.l2_hit_rate < 0.5 && self.l2_hits + self.l2_misses > 100 {
            tracing::warn!("L2命中率偏低，建议增加L2缓存TTL或预热常用数据");
        }
        
        if self.stampede_prevented > 100 {
            tracing::info!("成功防止{}次缓存击穿，请求合并机制工作正常", self.stampede_prevented);
        }
        
        if self.null_cache_hits > self.total_requests / 10 {
            tracing::warn!("空值缓存命中率较高，可能存在大量无效查询");
        }
    }
}
