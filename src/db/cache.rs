use crate::db::model::{Club, Heat, HeatRegistration, Race, Regatta, Registration};
use async_trait::async_trait;
use std::{hash::Hash, time::Duration};
use stretto::AsyncCache;
use tokio::task;

const MAX_COST: i64 = 1e6 as i64;

#[async_trait]
pub trait CacheTrait<K, V> {
    async fn get(&self, key: &K) -> Option<V>;
    async fn set(&self, key: &K, value: &V);
}

pub struct Cache<K, V>
where
    K: Hash + Eq + Send + Sync + Copy,
    V: Send + Sync + Clone + 'static,
{
    cache: AsyncCache<K, V>,
    ttl: Duration,
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Send + Sync + Copy,
    V: Send + Sync + Clone + 'static,
{
    pub fn new(size: usize, ttl: Duration) -> Self {
        Cache {
            cache: AsyncCache::new(size, MAX_COST, task::spawn).unwrap(),
            ttl,
        }
    }
}

#[async_trait]
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

    // caches with entries per race
    pub race: Cache<i32, Race>,
    pub club: Cache<i32, Club>,
    pub club_registrations: Cache<(i32, i32), Vec<Registration>>,
    pub regs: Cache<i32, Vec<Registration>>,

    // caches with entries per heat
    pub heat: Cache<i32, Heat>,
    pub heat_registrations: Cache<i32, Vec<HeatRegistration>>,
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

            // caches with entries per race
            race: Cache::new(MAX_RACES_COUNT, ttl),
            club: Cache::new(MAX_RACES_COUNT, ttl),
            club_registrations: Cache::new(MAX_RACES_COUNT, ttl),
            regs: Cache::new(MAX_RACES_COUNT, ttl),

            // ccaches with entries per heat
            heat: Cache::new(MAX_HEATS_COUNT, ttl),
            heat_registrations: Cache::new(MAX_HEATS_COUNT, ttl),
        }
    }
}
