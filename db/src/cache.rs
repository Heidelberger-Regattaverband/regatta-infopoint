use crate::aquarius::model::{Athlete, Club, Entry, Filters, Heat, Race, Regatta, Schedule};
use futures::future::join_all;
use log::{error, warn};
use std::{collections::HashMap, fmt::Display, hash::Hash, time::Duration};
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
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Send + Sync + Copy,
    V: Send + Sync + Clone + 'static,
{
    pub async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        match self.cache.get(key).await {
            Some(value_ref) => {
                let value = value_ref.value().clone();
                value_ref.release();
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub async fn set(&self, key: &K, value: &V) -> Result<(), CacheError> {
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

    pub async fn remove(&self, key: &K) -> Result<(), CacheError> {
        self.cache.remove(key).await;
        Ok(())
    }

    pub async fn clear(&self) -> Result<(), CacheError> {
        let _ = self.cache.clear().await;
        Ok(())
    }

    pub async fn stats(&self) -> CacheStats {
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

    /// Gets a value from the cache, or computes it using the provided function if not present.
    /// This implements the cache-aside pattern: check cache first, if miss then compute and store.
    ///
    /// # Arguments
    /// * `key` - The key to look up in the cache
    /// * `f` - An async function to compute the value if not found in cache
    ///
    /// # Returns
    /// The cached value or the newly computed value
    ///
    /// # Errors
    /// Returns `CacheError` if cache operations fail or if the computation function fails
    pub async fn compute_if_missing<F, Fut, E>(&self, key: &K, force: bool, f: F) -> Result<V, CacheError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<V, E>>,
        E: std::fmt::Display,
    {
        if force {
            // If force is true, skip cache and compute directly
            let value = f()
                .await
                .map_err(|e| CacheError::OperationFailed(format!("Computation failed: {}", e)))?;
            // Store in cache for future use
            self.set(key, &value).await?;
            Ok(value)
        } else {
            // First, try to get from cache
            match self.get(key).await? {
                Some(value) => Ok(value),
                None => {
                    // Cache miss - compute the value
                    let value = f()
                        .await
                        .map_err(|e| CacheError::OperationFailed(format!("Computation failed: {}", e)))?;

                    // Store in cache for future use
                    self.set(key, &value).await?;
                    Ok(value)
                }
            }
        }
    }
    pub async fn compute_if_missing_opt<F, Fut, E>(&self, key: &K, force: bool, f: F) -> Result<Option<V>, CacheError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Option<V>, E>>,
        E: Display,
    {
        if force {
            // If force is true, skip cache and compute directly
            let value = f()
                .await
                .map_err(|e| CacheError::OperationFailed(format!("Computation failed: {}", e)))?;
            // Store in cache for future use
            if let Some(v) = value.clone() {
                self.set(key, &v).await?;
            }
            Ok(value)
        } else {
            // First, try to get from cache
            match self.get(key).await? {
                Some(value) => Ok(Some(value)),
                None => {
                    // Cache miss - compute the value
                    let value = f()
                        .await
                        .map_err(|e| CacheError::OperationFailed(format!("Computation failed: {}", e)))?;

                    // Store in cache for future use
                    if let Some(v) = value.clone() {
                        self.set(key, &v).await?;
                    }
                    Ok(value)
                }
            }
        }
    }
}

/// Container for all caches with improved organization and error handling
pub struct Caches {
    // Caches with entries per regatta
    pub regattas: Cache<i32, Regatta>,
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
            regattas: Cache::try_new(config.regattas.clone())?,
            races: Cache::try_new(config.regattas.clone())?,
            heats: Cache::try_new(config.regattas.clone())?,
            clubs: Cache::try_new(config.regattas.clone())?,
            athletes: Cache::try_new(config.regattas.clone())?,
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

    /// Clears all caches
    pub async fn clear_all(&self) -> Result<(), CacheError> {
        // Clear all caches in parallel for better performance
        // Use boxed futures to ensure all futures have the same type
        use futures::future::FutureExt;

        let results = join_all(vec![
            self.regattas.clear().boxed(),
            self.races.clear().boxed(),
            self.heats.clear().boxed(),
            self.clubs.clear().boxed(),
            self.athletes.clear().boxed(),
            self.filters.clear().boxed(),
            self.schedule.clear().boxed(),
            self.club_with_aggregations.clear().boxed(),
            self.club_entries.clear().boxed(),
            self.athlete_entries.clear().boxed(),
            self.race_heats_entries.clear().boxed(),
            self.athlete.clear().boxed(),
            self.heat.clear().boxed(),
        ])
        .await;

        // Check if any operation failed
        for result in results {
            result?;
        }

        Ok(())
    }

    /// Gets statistics for all caches for monitoring purposes
    pub async fn get_all_stats(&self) -> HashMap<String, CacheStats> {
        let mut stats = HashMap::new();

        stats.insert("regatta".to_string(), self.regattas.stats().await);
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

/// Configuration for all caches in the system
#[derive(Debug, Clone)]
pub(crate) struct CachesConfig {
    pub(crate) regattas: CacheConfig,
    pub(crate) races: CacheConfig,
    pub(crate) heats: CacheConfig,
    pub(crate) clubs: CacheConfig,
    pub(crate) athletes: CacheConfig,
}

impl CachesConfig {
    pub(crate) fn new(base_ttl: Duration) -> Self {
        const MAX_REGATTAS_COUNT: usize = 3;
        const MAX_RACES_COUNT: usize = 200;
        const MAX_HEATS_COUNT: usize = 350;
        const MAX_CLUBS_COUNT: usize = 100;

        Self {
            regattas: CacheConfig {
                max_entries: MAX_REGATTAS_COUNT,
                ttl: base_ttl,
                max_cost: 100_000, // Smaller cost for regatta data
            },
            races: CacheConfig {
                max_entries: MAX_RACES_COUNT,
                ttl: base_ttl,
                max_cost: 500_000, // Medium cost for race data
            },
            heats: CacheConfig {
                max_entries: MAX_HEATS_COUNT,
                ttl: base_ttl,
                max_cost: 750_000, // Higher cost for heat data
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

/// Cache configuration to make cache behavior more configurable
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub(crate) max_entries: usize,
    /// Time-to-live for cache entries
    pub(crate) ttl: Duration,
    /// Maximum cost for the cache (memory limit)
    pub(crate) max_cost: i64,
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
        cache.remove(&1).await.unwrap();

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
        let config = CacheConfig {
            max_entries: 10,
            ttl: Duration::from_secs(60),
            max_cost: 1000,
        };
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

    #[tokio::test]
    async fn test_get_or_insert_with() {
        let config = CacheConfig {
            max_entries: 10,
            ttl: Duration::from_secs(60),
            max_cost: 1000,
        };
        let cache = Cache::<i32, String>::try_new(config).unwrap();

        // Test cache miss - should compute and store the value
        let result = cache
            .compute_if_missing(&1, false, || async {
                Ok::<String, &'static str>("computed_value".to_string())
            })
            .await
            .unwrap();
        assert_eq!(result, "computed_value");

        // Test cache hit - should return cached value without calling function
        let result = cache
            .compute_if_missing(&1, false, || async {
                Ok::<String, &'static str>("should_not_be_called".to_string())
            })
            .await
            .unwrap();
        assert_eq!(result, "computed_value"); // Should still be the original cached value

        // Verify the value is actually in cache
        let cached_value = cache.get(&1).await.unwrap();
        assert_eq!(cached_value, Some("computed_value".to_string()));
    }
}
