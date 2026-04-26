pub(crate) mod config;

use crate::aquarius::model::Notification;
use crate::aquarius::model::{Athlete, Club, Entry, Filters, Heat, Race, Regatta, Schedule};
use crate::cache::config::CacheConfig;
use crate::cache::config::CachesConfig;
use crate::error::DbError;
use ::futures::future::Future;
use ::std::any::type_name;
use ::std::fmt::Display;
use ::std::hash::Hash;

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
    fn try_new(config: CacheConfig) -> Result<Self, DbError> {
        let cache = AsyncCache::new(config.max_entries, config.max_entries as i64, task::spawn)?;
        debug!(type = type_name::<V>(), max_entries = config.max_entries, max_cost = config.max_entries, ttl = ?config.ttl,
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

    async fn get(&self, key: &K) -> Option<V> {
        match self.cache.get(key).await {
            Some(value_ref) => {
                let value = value_ref.value().clone();
                value_ref.release();
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(value)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    async fn set(&self, key: &K, value: &V) -> Result<bool, DbError> {
        // Insert with TTL and specified cost
        let result = self
            .cache
            .try_insert_with_ttl(*key, value.clone(), 1, self.config.ttl)
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
            match self.get(key).await {
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
            match self.get(key).await {
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
    pub(crate) athlete: Cache<(i32, i32), Athlete>,
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
        let all_stats = vec![
            self.regattas.stats(),
            self.races.stats(),
            self.heats.stats(),
            self.clubs.stats(),
            self.athletes.stats(),
            self.filters.stats(),
            self.schedule.stats(),
            self.club_with_aggregations.stats(),
            self.club_entries.stats(),
            self.athlete_entries.stats(),
            self.race_heats_entries.stats(),
            self.athlete.stats(),
            self.heat.stats(),
            self.notifications.stats(),
        ];

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
