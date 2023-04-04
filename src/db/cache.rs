use super::model::{Heat, HeatRegistration, Race, Regatta, Registration, Score};
use async_trait::async_trait;
use std::{hash::Hash, time::Duration};
use stretto::AsyncCache;
use tokio::task;

const MAX_COST: i64 = 1e6 as i64;

#[async_trait]
pub trait CacheTrait<K, V> {
    fn get(&self, key: &K) -> Option<V>;
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
    fn get(&self, key: &K) -> Option<V> {
        let opt_value_ref = self.cache.get(key);
        if let Some(value_ref) = opt_value_ref {
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
    pub regatta: Cache<i32, Regatta>,
    pub races: Cache<i32, Vec<Race>>,
    pub race: Cache<i32, Race>,
    pub regs: Cache<i32, Vec<Registration>>,
    pub heats: Cache<i32, Vec<Heat>>,
    pub heat_regs: Cache<i32, Vec<HeatRegistration>>,
    pub scores: Cache<i32, Vec<Score>>,
}

impl Caches {
    /// Creates a new `Cache`.
    pub(super) fn new(ttl: Duration) -> Self {
        const MAX_REGATTAS_COUNT: usize = 5;
        const MAX_RACES_COUNT: usize = 200;
        const MAX_HEATS_COUNT: usize = 350;

        Caches {
            regatta: Cache::new(MAX_REGATTAS_COUNT, ttl),
            races: Cache::new(MAX_REGATTAS_COUNT, ttl),
            race: Cache::new(MAX_RACES_COUNT, ttl),
            regs: Cache::new(MAX_RACES_COUNT, ttl),
            heats: Cache::new(MAX_REGATTAS_COUNT, ttl),
            heat_regs: Cache::new(MAX_HEATS_COUNT, ttl),
            scores: Cache::new(MAX_REGATTAS_COUNT, ttl),
        }
    }
}
