use aquarius::db::model::{Athlete, Club, Entry, Filters, Heat, Race, Regatta, Schedule};
use std::{hash::Hash, time::Duration};
use stretto::AsyncCache;
use tokio::task;

/// The maximum cost of the cache. This is used to limit the number of entries in the cache.
const MAX_COST: i64 = 1e6 as i64;

/// Trait for a cache. It is used to store and retrieve values from a cache.
pub trait CacheTrait<K, V> {
    /// Retrieves a value from the cache. If the value is not present in the cache, `None` is returned.
    /// # Arguments
    /// * `key` - The key of the value to retrieve.
    /// # Returns
    /// * `Some(value)` - The value associated with the key.
    /// * `None` - If the value is not present in the cache.
    async fn get(&self, key: &K) -> Option<V>;

    /// Sets a value in the cache.
    /// # Arguments
    /// * `key` - The key of the value to set.
    /// * `value` - The value to set.
    async fn set(&self, key: &K, value: &V);
}

/// A cache that uses `stretto` as the underlying cache.
pub struct Cache<K, V>
where
    K: Hash + Eq + Send + Sync + Copy,
    V: Send + Sync + Clone + 'static,
{
    /// The underlying cache.
    cache: AsyncCache<K, V>,

    /// The time-to-live of the entries in the cache.
    ttl: Duration,
}

/// Implementation of the `Cache` struct.
impl<K: Hash + Eq + Send + Sync + Copy, V> Cache<K, V>
where
    K: Hash + Eq + Send + Sync + Copy,
    V: Send + Sync + Clone + 'static,
{
    /// Creates a new `Cache`.
    /// # Arguments
    /// * `size` - The maximum number of entries in the cache.
    /// * `ttl` - The time-to-live of the entries in the cache.
    /// # Returns
    /// A new `Cache`.
    /// # Panics
    /// If the creation of the cache fails.
    pub fn new(size: usize, ttl: Duration) -> Self {
        Cache {
            cache: AsyncCache::new(size, MAX_COST, task::spawn).unwrap(),
            ttl,
        }
    }
}

impl<K, V> CacheTrait<K, V> for Cache<K, V>
where
    K: Hash + Eq + Send + Sync + Copy,
    V: Send + Sync + Clone + 'static,
{
    async fn get(&self, key: &K) -> Option<V> {
        let opt_value_ref = self.cache.get(key);
        if let Some(value_ref) = opt_value_ref.await {
            let value = value_ref.value().clone();
            value_ref.release();
            Some(value)
        } else {
            None
        }
    }

    async fn set(&self, key: &K, value: &V) {
        self.cache.insert_with_ttl(*key, value.clone(), 1, self.ttl).await;
        self.cache.wait().await.unwrap();
    }
}

pub(super) struct Caches {
    // caches with entries per regatta
    pub regatta: Cache<i32, Regatta>,
    pub races: Cache<i32, Vec<Race>>,
    pub heats: Cache<i32, Vec<Heat>>,
    pub participating_clubs: Cache<i32, Vec<Club>>,
    pub athletes: Cache<i32, Vec<Athlete>>,
    pub filters: Cache<i32, Filters>,
    pub schedule: Cache<i32, Schedule>,

    // caches with entries per race
    pub race_heats_entries: Cache<i32, Race>,
    pub club_entries: Cache<(i32, i32), Vec<Entry>>,
    pub athlete: Cache<i32, Athlete>,
    pub athlete_entries: Cache<(i32, i32), Vec<Entry>>,

    // caches with entries per heat
    pub heat: Cache<i32, Heat>,
}

impl Caches {
    /// Creates a new `Cache`.
    pub fn new(ttl: Duration) -> Self {
        const MAX_REGATTAS_COUNT: usize = 3;
        const MAX_RACES_COUNT: usize = 200;
        const MAX_HEATS_COUNT: usize = 350;

        Caches {
            // caches with entries per regatta
            regatta: Cache::new(MAX_REGATTAS_COUNT, ttl),
            races: Cache::new(MAX_REGATTAS_COUNT, ttl),
            heats: Cache::new(MAX_REGATTAS_COUNT, ttl),
            participating_clubs: Cache::new(MAX_REGATTAS_COUNT, ttl),
            athletes: Cache::new(MAX_REGATTAS_COUNT, ttl),
            filters: Cache::new(MAX_REGATTAS_COUNT, ttl),
            schedule: Cache::new(MAX_REGATTAS_COUNT, ttl),

            // caches with entries per race
            race_heats_entries: Cache::new(MAX_RACES_COUNT, ttl),
            club_entries: Cache::new(MAX_RACES_COUNT, ttl),
            athlete: Cache::new(MAX_RACES_COUNT, ttl),
            athlete_entries: Cache::new(MAX_RACES_COUNT, ttl),

            // caches with entries per heat
            heat: Cache::new(MAX_HEATS_COUNT, ttl),
        }
    }
}
