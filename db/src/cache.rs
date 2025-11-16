use crate::{
    aquarius::model::{Athlete, Club, Entry, Filters, Heat, Race, Regatta, Schedule},
    error::DbError,
};
use futures::future::Future;
use std::{
    fmt::Display,
    hash::Hash,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use stretto::AsyncCache;
use tokio::task;
use tracing::{debug, warn};

/// A high-performance cache that uses `stretto` as the underlying cache with comprehensive features
///
/// This cache provides:
/// - Automatic metrics tracking (hits, misses, hit rate)
/// - TTL support with configurable expiration
/// - Cost-based eviction policies
/// - Thread-safe operations
/// - Graceful error handling
/// - Cache-aside pattern support
pub struct Cache<K, V>
where
    K: Hash + Eq + Send + Sync + Copy + 'static,
    V: Send + Sync + Clone + 'static,
{
    /// The underlying stretto cache
    cache: AsyncCache<K, V>,
    /// Cache configuration
    config: CacheConfig,
    /// Atomic counter for cache hits
    hits: AtomicU64,
    /// Atomic counter for cache misses
    misses: AtomicU64,
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Send + Sync + Copy + 'static,
    V: Send + Sync + Clone + 'static,
{
    pub fn try_new(config: CacheConfig) -> Result<Self, DbError> {
        let cache = AsyncCache::new(config.max_entries, config.max_cost, task::spawn)
            .map_err(|e| DbError::Cache(format!("Failed to create cache: {}", e)))?;
        debug!(
            "Created cache with max_entries: {}, max_cost: {}, ttl: {:?}",
            config.max_entries, config.max_cost, config.ttl
        );
        Ok(Cache {
            cache,
            config,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        })
    }

    fn stats(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);

        CacheStats {
            hits,
            misses,
            entries: self.cache.len(),
            hit_rate: if hits + misses > 0 {
                (hits as f64 / (hits + misses) as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    async fn get(&self, key: &K) -> Result<Option<V>, DbError> {
        match self.cache.get(key).await {
            Some(value_ref) => {
                let value = value_ref.value().clone();
                value_ref.release();
                self.hits.fetch_add(1, Ordering::Relaxed);
                Ok(Some(value))
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                Ok(None)
            }
        }
    }

    async fn set(&self, key: &K, value: &V) -> Result<(), DbError> {
        self.set_with_cost(key, value, 1).await
    }

    async fn set_with_cost(&self, key: &K, value: &V, cost: i64) -> Result<(), DbError> {
        // Insert with TTL and specified cost
        self.cache
            .insert_with_ttl(*key, value.clone(), cost, self.config.ttl)
            .await;

        // Handle the wait operation more gracefully
        if let Err(e) = self.cache.wait().await {
            warn!("Cache wait operation failed: {}", e);
            return Err(DbError::Cache(format!("Cache wait failed: {}", e)));
        }
        Ok(())
    }

    pub async fn compute_if_missing<F, Fut, E>(&self, key: &K, force: bool, f: F) -> Result<V, DbError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<V, E>>,
        E: Display,
    {
        if force {
            let value = f()
                .await
                .map_err(|e| DbError::Cache(format!("Computation failed: {}", e)))?;
            self.set(key, &value).await?;
            Ok(value)
        } else {
            match self.get(key).await? {
                Some(value) => Ok(value),
                None => {
                    let value = f()
                        .await
                        .map_err(|e| DbError::Cache(format!("Computation failed: {}", e)))?;
                    self.set(key, &value).await?;
                    Ok(value)
                }
            }
        }
    }

    pub async fn compute_if_missing_opt<F, Fut, E>(&self, key: &K, force: bool, f: F) -> Result<Option<V>, DbError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Option<V>, E>>,
        E: Display,
    {
        if force {
            let value = f()
                .await
                .map_err(|e| DbError::Cache(format!("Computation failed: {}", e)))?;
            // Only cache non-None values
            if let Some(ref v) = value {
                self.set(key, v).await?;
            }
            Ok(value)
        } else {
            match self.get(key).await? {
                Some(value) => Ok(Some(value)),
                None => {
                    let value = f()
                        .await
                        .map_err(|e| DbError::Cache(format!("Computation failed: {}", e)))?;
                    // Only cache non-None values
                    if let Some(ref v) = value {
                        self.set(key, v).await?;
                    }
                    Ok(value)
                }
            }
        }
    }
}

/// Container for all caches with improved organization, better error handling, and type safety
///
/// This struct organizes caches by their usage patterns:
/// - Per-regatta caches for regatta-scoped data
/// - Composite key caches for entity relationships  
/// - Individual entity caches for direct lookups
pub struct Caches {
    // Caches with entries per regatta
    pub regattas: Cache<i32, Regatta>,
    pub races: Cache<i32, Vec<Race>>,
    pub heats: Cache<i32, Vec<Heat>>,
    pub clubs: Cache<i32, Vec<Club>>,
    pub athletes: Cache<i32, Vec<Athlete>>,
    pub filters: Cache<i32, Filters>,
    pub schedule: Cache<i32, Schedule>,

    // Cachesq with composite keys (regatta_id, entity_id)
    pub club_with_aggregations: Cache<(i32, i32), Club>,
    pub club_entries: Cache<(i32, i32), Vec<Entry>>,
    pub athlete_entries: Cache<(i32, i32), Vec<Entry>>,

    // Caches with entries per race/heat/athlete
    pub race_heats_entries: Cache<i32, Race>,
    pub athlete: Cache<i32, Athlete>,
    pub heat: Cache<i32, Heat>,
}

impl Caches {
    pub fn try_new(ttl: Duration) -> Result<Self, DbError> {
        let config = CachesConfig::new(ttl);

        Ok(Caches {
            // Caches with entries per regatta - using regatta config for all regatta-scoped data
            regattas: Cache::try_new(config.regattas.clone())?,
            races: Cache::try_new(config.races.clone())?,
            heats: Cache::try_new(config.heats.clone())?,
            clubs: Cache::try_new(config.clubs.clone())?,
            athletes: Cache::try_new(config.athletes.clone())?,
            filters: Cache::try_new(config.regattas.clone())?,
            schedule: Cache::try_new(config.regattas)?,

            // Caches with composite keys
            club_with_aggregations: Cache::try_new(config.clubs.clone())?,
            club_entries: Cache::try_new(config.clubs)?,
            athlete_entries: Cache::try_new(config.athletes.clone())?,

            // Caches with entries per race/heat/athlete
            race_heats_entries: Cache::try_new(config.races)?,
            athlete: Cache::try_new(config.athletes)?,
            heat: Cache::try_new(config.heats)?,
        })
    }

    pub fn get_summary_stats(&self) -> CacheStats {
        let all_stats = {
            let this = &self;
            vec![
                this.regattas.stats(),
                this.races.stats(),
                this.heats.stats(),
                this.clubs.stats(),
                this.athletes.stats(),
                this.filters.stats(),
                this.schedule.stats(),
                this.club_with_aggregations.stats(),
                this.club_entries.stats(),
                this.athlete_entries.stats(),
                this.race_heats_entries.stats(),
                this.athlete.stats(),
                this.heat.stats(),
            ]
        };

        let mut total_hits = 0;
        let mut total_misses = 0;
        let mut total_entries = 0;

        for stat in all_stats {
            total_hits += stat.hits;
            total_misses += stat.misses;
            total_entries += stat.entries;
        }

        let stats = CacheStats {
            hits: total_hits,
            misses: total_misses,
            entries: total_entries,
            hit_rate: if total_hits + total_misses > 0 {
                (total_hits as f64 / (total_hits + total_misses) as f64) * 100.0
            } else {
                0.0
            },
        };
        debug!("Cache statistics: {:?}", stats);
        stats
    }
}

/// Configuration for all caches in the system with optimized defaults
#[derive(Debug, Clone)]
pub(crate) struct CachesConfig {
    pub(crate) regattas: CacheConfig,
    pub(crate) races: CacheConfig,
    pub(crate) heats: CacheConfig,
    pub(crate) clubs: CacheConfig,
    pub(crate) athletes: CacheConfig,
}

impl CachesConfig {
    /// Creates cache configurations with optimized settings for each data type
    ///
    /// # Arguments
    /// * `base_ttl` - Base time-to-live applied to all caches
    ///
    /// # Returns
    /// Configured cache settings optimized for regatta data patterns
    pub(crate) fn new(base_ttl: Duration) -> Self {
        // Constants based on typical regatta sizes and usage patterns
        const MAX_REGATTAS_COUNT: usize = 3;
        const MAX_RACES_COUNT: usize = 200;
        const MAX_HEATS_COUNT: usize = 350;
        const MAX_CLUBS_COUNT: usize = 100;

        Self {
            regattas: CacheConfig {
                max_entries: MAX_REGATTAS_COUNT,
                ttl: base_ttl,
                max_cost: 100_000, // Lower cost - regattas are small but critical
            },
            races: CacheConfig {
                max_entries: MAX_RACES_COUNT,
                ttl: base_ttl,
                max_cost: 500_000, // Medium cost for race data with results
            },
            heats: CacheConfig {
                max_entries: MAX_HEATS_COUNT,
                ttl: base_ttl,
                max_cost: 750_000, // Higher cost - heats contain entry lists
            },
            clubs: CacheConfig {
                max_entries: MAX_CLUBS_COUNT,
                ttl: base_ttl,
                max_cost: 200_000, // Medium cost for club data
            },
            athletes: CacheConfig {
                max_entries: MAX_RACES_COUNT, // Reuse race count for athletes
                ttl: base_ttl,
                max_cost: 300_000, // Medium cost for athlete data
            },
        }
    }
}

/// Cache configuration with builder pattern support
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub(crate) max_entries: usize,
    /// Time-to-live for cache entries
    pub(crate) ttl: Duration,
    /// Maximum cost for the cache (memory limit)
    pub(crate) max_cost: i64,
}

/// Cache statistics for monitoring and debugging with actual tracking capabilities
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
    pub hit_rate: f64,
}
