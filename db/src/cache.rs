use crate::aquarius::model::Notification;
use crate::aquarius::model::{Athlete, Club, Entry, Filters, Heat, Race, Regatta, Schedule};
use crate::error::DbError;
use ::futures::future::Future;
use ::std::any::type_name;
use ::std::fmt::Display;
use ::std::hash::Hash;
use ::std::mem;
use ::std::sync::atomic::{AtomicU64, Ordering};
use ::std::time::Duration;
use ::stretto::AsyncCache;
use ::tokio::task;
use ::tracing::debug;

/// A high-performance cache that uses `stretto` as the underlying cache with comprehensive features
///
/// This cache provides:
/// - Automatic metrics tracking (hits, misses, hit rate)
/// - TTL support with configurable expiration
/// - Cost-based eviction policies
/// - Thread-safe operations
/// - Graceful error handling
/// - Cache-aside pattern support
pub(crate) struct Cache<K, V>
where
    K: Hash + Eq + Send + Sync + Copy + 'static,
    V: Send + Sync + Clone + CacheCost + 'static,
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
    V: Send + Sync + Clone + CacheCost + 'static,
{
    fn try_new(config: CacheConfig) -> Result<Self, DbError> {
        let cache = AsyncCache::new(config.max_entries, config.max_cost as i64, task::spawn)?;
        debug!(type = type_name::<V>(), max_entries = config.max_entries, max_cost = config.max_cost, ttl = ?config.ttl,
            "New Cache:"
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

    async fn set(&self, key: &K, value: &V) -> Result<bool, DbError> {
        let cost = value.cache_cost();
        self.set_with_cost(key, value, cost).await
    }

    async fn set_with_cost(&self, key: &K, value: &V, cost: i64) -> Result<bool, DbError> {
        // Insert with TTL and specified cost
        let result = self
            .cache
            .try_insert_with_ttl(*key, value.clone(), cost, self.config.ttl)
            .await?;
        Ok(result)
    }

    pub(crate) async fn compute_if_missing<F, Fut, E>(&self, key: &K, force: bool, f: F) -> Result<V, DbError>
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

    pub(crate) async fn compute_if_missing_opt<F, Fut, E>(
        &self,
        key: &K,
        force: bool,
        f: F,
    ) -> Result<Option<V>, DbError>
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

    pub(crate) async fn invalidate(&self, key: &K) -> Result<(), DbError> {
        self.cache.try_remove(key).await.map_err(DbError::CacheError)
    }
}

/// Trait for estimating the memory cost of a cached value.
///
/// Used by the cache to assign a meaningful cost for admission and eviction policies.
/// Implementations should estimate the total memory footprint including heap allocations.
pub(crate) trait CacheCost {
    /// Returns the estimated memory cost in bytes.
    fn cache_cost(&self) -> i64;
}

/// Blanket implementation for `Vec<T>` that accounts for heap-allocated elements.
/// The cost includes the `Vec` stack overhead plus the estimated cost of each element.
impl<T: CacheCost> CacheCost for Vec<T> {
    fn cache_cost(&self) -> i64 {
        let stack = mem::size_of::<Vec<T>>() as i64;
        let heap: i64 = self.iter().map(|item| item.cache_cost()).sum();
        stack + heap
    }
}

/// Implements `CacheCost` for types where `mem::size_of` is a reasonable approximation.
/// This covers model structs whose heap-allocated fields (e.g. `String`) are relatively small.
macro_rules! impl_cache_cost {
    ($($ty:ty),*) => {
        $(
            impl CacheCost for $ty {
                fn cache_cost(&self) -> i64 {
                    mem::size_of::<$ty>() as i64
                }
            }
        )*
    };
}

impl_cache_cost!(
    Regatta,
    Race,
    Heat,
    Club,
    Athlete,
    Entry,
    Notification,
    Filters,
    Schedule
);

/// Container for all caches with improved organization, better error handling, and type safety
///
/// This struct organizes caches by their usage patterns:
/// - Per-regatta caches for regatta-scoped data
/// - Composite key caches for entity relationships  
/// - Individual entity caches for direct lookups
pub(crate) struct Caches {
    // Caches with entries per regatta
    pub(crate) regattas: Cache<i32, Regatta>,
    pub(crate) races: Cache<i32, Vec<Race>>,
    pub(crate) heats: Cache<i32, Vec<Heat>>,
    pub(crate) clubs: Cache<i32, Vec<Club>>,
    pub(crate) athletes: Cache<i32, Vec<Athlete>>,
    pub(crate) filters: Cache<i32, Filters>,
    pub(crate) schedule: Cache<i32, Schedule>,

    // Caches with composite keys (regatta_id, entity_id)
    pub(crate) club_with_aggregations: Cache<(i32, i32), Club>,
    pub(crate) club_entries: Cache<(i32, i32), Vec<Entry>>,
    pub(crate) athlete_entries: Cache<(i32, i32), Vec<Entry>>,

    // Caches with entries per race/heat/athlete
    pub(crate) race_heats_entries: Cache<i32, Race>,
    pub(crate) athlete: Cache<i32, Athlete>,
    pub(crate) heat: Cache<i32, Heat>,

    pub(crate) notifications: Cache<i32, Vec<Notification>>,
}

impl Caches {
    pub(crate) fn try_new(ttl: Duration) -> Result<Self, DbError> {
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

            notifications: Cache::try_new(config.notifications)?,
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
                this.notifications.stats(),
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

        CacheStats {
            hits: total_hits,
            misses: total_misses,
            entries: total_entries,
            hit_rate: if total_hits + total_misses > 0 {
                (total_hits as f64 / (total_hits + total_misses) as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Configuration for all caches in the system with optimized defaults
#[derive(Debug, Clone)]
struct CachesConfig {
    regattas: CacheConfig,
    races: CacheConfig,
    heats: CacheConfig,
    clubs: CacheConfig,
    athletes: CacheConfig,
    notifications: CacheConfig,
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
        const MAX_NOTIFICATIONS_COUNT: usize = 10;
        Self {
            regattas: CacheConfig {
                max_entries: MAX_REGATTAS_COUNT,
                ttl: base_ttl,
                max_cost: mem::size_of::<Regatta>() * MAX_REGATTAS_COUNT,
            },
            races: CacheConfig {
                max_entries: MAX_RACES_COUNT,
                ttl: base_ttl,
                max_cost: mem::size_of::<Race>() * MAX_RACES_COUNT,
            },
            heats: CacheConfig {
                max_entries: MAX_HEATS_COUNT,
                ttl: base_ttl,
                max_cost: mem::size_of::<Heat>() * MAX_HEATS_COUNT,
            },
            clubs: CacheConfig {
                max_entries: MAX_CLUBS_COUNT,
                ttl: base_ttl,
                max_cost: mem::size_of::<Club>() * MAX_CLUBS_COUNT,
            },
            athletes: CacheConfig {
                max_entries: MAX_RACES_COUNT,
                ttl: base_ttl,
                max_cost: mem::size_of::<Athlete>() * MAX_RACES_COUNT,
            },
            notifications: CacheConfig {
                max_entries: MAX_NOTIFICATIONS_COUNT,
                ttl: base_ttl,
                max_cost: mem::size_of::<Notification>() * MAX_NOTIFICATIONS_COUNT,
            },
        }
    }
}

/// Cache configuration with builder pattern support
#[derive(Debug, Clone)]
struct CacheConfig {
    /// Maximum number of entries in the cache
    max_entries: usize,
    /// Time-to-live for cache entries
    ttl: Duration,
    /// Maximum cost for the cache (memory limit)
    max_cost: usize,
}

/// Cache statistics for monitoring and debugging with actual tracking capabilities
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of cache hits
    pub hits: u64,
    /// Total number of cache misses
    pub misses: u64,
    /// Current number of entries in the cache
    pub entries: usize,
    /// Cache hit rate as a percentage
    pub hit_rate: f64,
}
