use db::aquarius::model::{Athlete, Club, Entry, Filters, Heat, Race, Regatta, Schedule};
use log::{error, warn};
use std::{hash::Hash, time::Duration};
use stretto::AsyncCache;
use thiserror::Error;
use tokio::task;

/// Cache-specific error types for better error handling
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Failed to create cache with size {size}: {source}")]
    CreationFailed { size: usize, source: stretto::CacheError },
    #[error("Cache operation timed out")]
    Timeout,
    #[error("Cache operation failed: {0}")]
    OperationFailed(String),
}

/// Cache configuration to make cache behavior more configurable
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_entries: usize,
    /// Time-to-live for cache entries
    pub ttl: Duration,
    /// Maximum cost for the cache (memory limit)
    pub max_cost: i64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl: Duration::from_secs(300), // 5 minutes
            max_cost: 1_000_000,
        }
    }
}

/// Trait for a cache with improved error handling and additional methods
pub trait CacheTrait<K, V> {
    /// Retrieves a value from the cache
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError>;

    /// Sets a value in the cache
    async fn set(&self, key: &K, value: &V) -> Result<(), CacheError>;

    /// Removes a value from the cache
    async fn remove(&self, key: &K) -> Result<bool, CacheError>;

    /// Clears all entries from the cache
    async fn clear(&self) -> Result<(), CacheError>;

    /// Returns cache statistics (hit rate, entry count, etc.)
    async fn stats(&self) -> CacheStats;
}

/// Cache statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
    pub hit_rate: f64,
}

/// A cache that uses `stretto` as the underlying cache with improved error handling
pub struct Cache<K, V>
where
    K: Hash + Eq + Send + Sync + Copy,
    V: Send + Sync + Clone + 'static,
{
    /// The underlying cache
    cache: AsyncCache<K, V>,
    /// Cache configuration
    config: CacheConfig,
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Send + Sync + Copy,
    V: Send + Sync + Clone + 'static,
{
    /// Creates a new `Cache` with the given configuration
    /// Returns a Result instead of panicking
    pub fn try_new(config: CacheConfig) -> Result<Self, CacheError> {
        let cache = AsyncCache::new(config.max_entries, config.max_cost, task::spawn).map_err(|e| {
            CacheError::CreationFailed {
                size: config.max_entries,
                source: e,
            }
        })?;

        Ok(Cache { cache, config })
    }

    /// Creates a new `Cache` with default configuration that panics on failure
    /// Kept for backward compatibility but logs the error before panicking
    pub fn new(size: usize, ttl: Duration) -> Self {
        let config = CacheConfig {
            max_entries: size,
            ttl,
            max_cost: 1_000_000,
        };

        Self::try_new(config).unwrap_or_else(|e| {
            error!("Failed to create cache: {}", e);
            panic!(
                "Critical error: Cannot create cache - application cannot continue: {}",
                e
            );
        })
    }

    /// Creates a new `Cache` with custom configuration
    pub fn with_config(config: CacheConfig) -> Result<Self, CacheError> {
        Self::try_new(config)
    }

    /// Returns the cache configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
}

impl<K, V> CacheTrait<K, V> for Cache<K, V>
where
    K: Hash + Eq + Send + Sync + Copy,
    V: Send + Sync + Clone + 'static,
{
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        match self.cache.get(key).await {
            Some(value_ref) => {
                let value = value_ref.value().clone();
                value_ref.release();
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn set(&self, key: &K, value: &V) -> Result<(), CacheError> {
        // Insert with TTL and cost of 1
        self.cache
            .insert_with_ttl(*key, value.clone(), 1, self.config.ttl)
            .await;

        // Handle the wait operation more gracefully
        if let Err(e) = self.cache.wait().await {
            warn!("Cache wait operation failed: {}", e);
            // Return error instead of just logging for better error propagation
            return Err(CacheError::OperationFailed(format!("Wait failed: {}", e)));
        }

        Ok(())
    }

    async fn remove(&self, key: &K) -> Result<bool, CacheError> {
        self.cache.remove(key).await;
        // stretto's remove doesn't return whether the key existed,
        // so we always return true to indicate the operation completed
        Ok(true)
    }

    async fn clear(&self) -> Result<(), CacheError> {
        let _ = self.cache.clear().await;
        Ok(())
    }

    async fn stats(&self) -> CacheStats {
        // Since stretto's AsyncCache doesn't expose metrics directly,
        // we'll return basic stats. In a real implementation, you might
        // want to track these metrics manually or use a different cache library
        // that provides better observability.
        CacheStats {
            hits: 0,   // Would need to be tracked manually
            misses: 0, // Would need to be tracked manually
            entries: self.cache.len(),
            hit_rate: 0.0, // Would need to be calculated from tracked metrics
        }
    }
}

/// Configuration for all caches in the system
#[derive(Debug, Clone)]
pub struct CachesConfig {
    pub regatta_cache: CacheConfig,
    pub race_cache: CacheConfig,
    pub heat_cache: CacheConfig,
    pub club_cache: CacheConfig,
    pub athlete_cache: CacheConfig,
}

impl CachesConfig {
    pub fn new(base_ttl: Duration) -> Self {
        const MAX_REGATTAS_COUNT: usize = 3;
        const MAX_RACES_COUNT: usize = 200;
        const MAX_HEATS_COUNT: usize = 350;
        const MAX_CLUBS_COUNT: usize = 100;

        Self {
            regatta_cache: CacheConfig {
                max_entries: MAX_REGATTAS_COUNT,
                ttl: base_ttl,
                max_cost: 100_000, // Smaller cost for regatta data
            },
            race_cache: CacheConfig {
                max_entries: MAX_RACES_COUNT,
                ttl: base_ttl,
                max_cost: 500_000, // Medium cost for race data
            },
            heat_cache: CacheConfig {
                max_entries: MAX_HEATS_COUNT,
                ttl: base_ttl,
                max_cost: 750_000, // Higher cost for heat data
            },
            club_cache: CacheConfig {
                max_entries: MAX_CLUBS_COUNT,
                ttl: base_ttl,
                max_cost: 200_000, // Medium cost for club data
            },
            athlete_cache: CacheConfig {
                max_entries: MAX_RACES_COUNT, // Reuse race count for athletes
                ttl: base_ttl,
                max_cost: 300_000, // Medium cost for athlete data
            },
        }
    }
}

/// Container for all caches with improved organization and error handling
pub(super) struct Caches {
    // Caches with entries per regatta
    pub regatta: Cache<i32, Regatta>,
    pub races: Cache<i32, Vec<Race>>,
    pub heats: Cache<i32, Vec<Heat>>,
    pub clubs: Cache<i32, Vec<Club>>,
    pub athletes: Cache<i32, Vec<Athlete>>,
    pub filters: Cache<i32, Filters>,
    pub schedule: Cache<i32, Schedule>,

    // Caches with composite keys (regatta_id, entity_id)
    pub club_with_aggregations: Cache<(i32, i32), Club>,
    pub club_entries: Cache<(i32, i32), Vec<Entry>>,
    pub athlete_entries: Cache<(i32, i32), Vec<Entry>>,

    // Caches with entries per race/heat/athlete
    pub race_heats_entries: Cache<i32, Race>,
    pub athlete: Cache<i32, Athlete>,
    pub heat: Cache<i32, Heat>,
}

impl Caches {
    /// Creates a new `Caches` instance with the given TTL
    /// Now returns a Result for better error handling
    pub fn try_new(ttl: Duration) -> Result<Self, CacheError> {
        let config = CachesConfig::new(ttl);

        Ok(Caches {
            // Caches with entries per regatta
            regatta: Cache::try_new(config.regatta_cache.clone())?,
            races: Cache::try_new(config.regatta_cache.clone())?,
            heats: Cache::try_new(config.regatta_cache.clone())?,
            clubs: Cache::try_new(config.regatta_cache.clone())?,
            athletes: Cache::try_new(config.regatta_cache.clone())?,
            filters: Cache::try_new(config.regatta_cache.clone())?,
            schedule: Cache::try_new(config.regatta_cache)?,

            // Caches with composite keys
            club_with_aggregations: Cache::try_new(config.club_cache.clone())?,
            club_entries: Cache::try_new(config.club_cache)?,
            athlete_entries: Cache::try_new(config.athlete_cache.clone())?,

            // Caches with entries per race/heat/athlete
            race_heats_entries: Cache::try_new(config.race_cache)?,
            athlete: Cache::try_new(config.athlete_cache)?,
            heat: Cache::try_new(config.heat_cache)?,
        })
    }

    /// Clears all caches
    pub async fn clear_all(&self) -> Result<(), CacheError> {
        // Clear all caches in parallel for better performance
        let results = tokio::join!(
            self.regatta.clear(),
            self.races.clear(),
            self.heats.clear(),
            self.clubs.clear(),
            self.athletes.clear(),
            self.filters.clear(),
            self.schedule.clear(),
            self.club_with_aggregations.clear(),
            self.club_entries.clear(),
            self.athlete_entries.clear(),
            self.race_heats_entries.clear(),
            self.athlete.clear(),
            self.heat.clear()
        );

        // Check if any operation failed
        for result in [
            results.0, results.1, results.2, results.3, results.4, results.5, results.6, results.7, results.8,
            results.9, results.10, results.11, results.12,
        ] {
            result?;
        }

        Ok(())
    }

    /// Gets statistics for all caches for monitoring purposes
    pub async fn get_all_stats(&self) -> std::collections::HashMap<String, CacheStats> {
        let mut stats = std::collections::HashMap::new();

        stats.insert("regatta".to_string(), self.regatta.stats().await);
        stats.insert("races".to_string(), self.races.stats().await);
        stats.insert("heats".to_string(), self.heats.stats().await);
        stats.insert("clubs".to_string(), self.clubs.stats().await);
        stats.insert("athletes".to_string(), self.athletes.stats().await);
        stats.insert("filters".to_string(), self.filters.stats().await);
        stats.insert("schedule".to_string(), self.schedule.stats().await);
        stats.insert(
            "club_with_aggregations".to_string(),
            self.club_with_aggregations.stats().await,
        );
        stats.insert("club_entries".to_string(), self.club_entries.stats().await);
        stats.insert("athlete_entries".to_string(), self.athlete_entries.stats().await);
        stats.insert("race_heats_entries".to_string(), self.race_heats_entries.stats().await);
        stats.insert("athlete".to_string(), self.athlete.stats().await);
        stats.insert("heat".to_string(), self.heat.stats().await);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let config = CacheConfig {
            max_entries: 10,
            ttl: Duration::from_secs(60),
            max_cost: 1000,
        };

        let cache = Cache::<i32, String>::try_new(config).unwrap();

        // Test set and get
        cache.set(&1, &"value1".to_string()).await.unwrap();
        let result = cache.get(&1).await.unwrap();
        assert_eq!(result, Some("value1".to_string()));

        // Test remove
        let removed = cache.remove(&1).await.unwrap();
        assert!(removed);

        let result = cache.get(&1).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_caches_creation() {
        let caches = Caches::try_new(Duration::from_secs(300));
        assert!(caches.is_ok());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let config = CacheConfig::default();
        let cache = Cache::<i32, String>::try_new(config).unwrap();

        // Initially empty cache
        let stats = cache.stats().await;
        assert_eq!(stats.entries, 0);

        // Add an entry
        cache.set(&1, &"test".to_string()).await.unwrap();

        // Check that entry count increased
        let stats = cache.stats().await;
        assert_eq!(stats.entries, 1);
    }
}
