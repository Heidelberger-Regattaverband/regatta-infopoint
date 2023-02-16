use super::model::{
    heat::Heat, heat::HeatRegistration, race::Race, regatta::Regatta, registration::Registration,
    score::Score,
};
use async_std::task;
use async_trait::async_trait;
use log::{debug, trace};
use std::time::Duration;
use stretto::AsyncCache;

const MAX_COST: i64 = 1e6 as i64;

#[async_trait]
pub trait CacheTrait<K, T> {
    fn get(&self, key: &K) -> Option<T>;

    async fn set(&self, key: &K, value: &T);
}

pub struct RegattaCache {
    cache: AsyncCache<i32, Regatta>,
}
impl RegattaCache {
    pub fn new(size: usize) -> Self {
        RegattaCache {
            cache: AsyncCache::new(size, MAX_COST, task::spawn).unwrap(),
        }
    }
}

#[async_trait]
impl CacheTrait<i32, Regatta> for RegattaCache {
    fn get(self: &Self, key: &i32) -> Option<Regatta> {
        let opt_value_ref = self.cache.get(&key);
        if let Some(value_ref) = opt_value_ref {
            let value = value_ref.value().clone();
            value_ref.release();
            Some(value)
        } else {
            None
        }
    }

    async fn set(&self, key: &i32, value: &Regatta) {
        self.cache
            .insert_with_ttl(*key, value.clone(), 1, TTL)
            .await;
        self.cache.wait().await.unwrap();
    }
}

pub(super) struct Cache {
    pub regatta: RegattaCache,
    races: AsyncCache<i32, Vec<Race>>,
    race: AsyncCache<i32, Race>,
    regs: AsyncCache<i32, Vec<Registration>>,
    heats: AsyncCache<i32, Vec<Heat>>,
    heat_regs: AsyncCache<i32, Vec<HeatRegistration>>,
    scores: AsyncCache<i32, Vec<Score>>,
}

const TTL: Duration = Duration::from_secs(30);

impl Cache {
    /// Creates a new `Cache`.
    pub(super) fn new() -> Self {
        const MAX_REGATTAS_COUNT: usize = 5;
        const MAX_RACES_COUNT: usize = 200;
        const MAX_HEATS_COUNT: usize = 350;

        Cache {
            regatta: RegattaCache::new(MAX_REGATTAS_COUNT),
            races: AsyncCache::new(MAX_REGATTAS_COUNT, MAX_COST, task::spawn).unwrap(),
            race: AsyncCache::new(MAX_RACES_COUNT, MAX_COST, task::spawn).unwrap(),
            regs: AsyncCache::new(MAX_RACES_COUNT, MAX_COST, task::spawn).unwrap(),
            heats: AsyncCache::new(MAX_REGATTAS_COUNT, MAX_COST, task::spawn).unwrap(),
            heat_regs: AsyncCache::new(MAX_HEATS_COUNT, MAX_COST, task::spawn).unwrap(),
            scores: AsyncCache::new(MAX_REGATTAS_COUNT, MAX_COST, task::spawn).unwrap(),
        }
    }

    // heats

    pub(super) async fn insert_heats(&self, regatta_id: i32, heats: &[Heat]) {
        self.heats
            .insert_with_ttl(regatta_id, heats.to_owned(), 1, TTL)
            .await;
        self.heats.wait().await.unwrap();
    }

    pub(super) fn get_heats(&self, regatta_id: i32) -> Option<Vec<Heat>> {
        let opt_value_ref = self.heats.get(&regatta_id);
        // see also: https://doc.rust-lang.org/rust-by-example/flow_control/if_let.html
        if let Some(value_ref) = opt_value_ref {
            let value = value_ref.value().clone();
            value_ref.release();
            debug!("Reading heats of regatta {} from cache.", regatta_id);
            trace!("From cache: {:?}", value);
            return Some(value);
        }
        None
    }

    // heat_registrations
    pub async fn insert_heat_regs(&self, heat_id: i32, heat_regs: &[HeatRegistration]) {
        self.heat_regs
            .insert_with_ttl(heat_id, heat_regs.to_owned(), 1, TTL)
            .await;
        self.heat_regs.wait().await.unwrap();
    }

    pub fn get_heat_regs(&self, heat_id: i32) -> Option<Vec<HeatRegistration>> {
        let opt_value_ref = self.heat_regs.get(&heat_id);
        // see also: https://doc.rust-lang.org/rust-by-example/flow_control/if_let.html
        if let Some(value_ref) = opt_value_ref {
            let value = value_ref.value().clone();
            value_ref.release();
            debug!("Reading registrations of heat {} from cache.", heat_id);
            trace!("From cache: {:?}", value);
            return Some(value);
        }
        None
    }

    // scores
    pub(super) async fn insert_scores(&self, regatta_id: i32, scores: &[Score]) {
        self.scores
            .insert_with_ttl(regatta_id, scores.to_owned(), 1, TTL)
            .await;
        self.scores.wait().await.unwrap();
    }

    pub(super) fn get_scores(&self, regatta_id: i32) -> Option<Vec<Score>> {
        let opt_value_ref = self.scores.get(&regatta_id);
        // see also: https://doc.rust-lang.org/rust-by-example/flow_control/if_let.html
        if let Some(value_ref) = opt_value_ref {
            let value = value_ref.value().clone();
            value_ref.release();
            debug!("Reading scores of regatta {} from cache.", regatta_id);
            trace!("From cache: {:?}", value);
            return Some(value);
        }
        None
    }

    // races
    pub(super) async fn insert_races(&self, regatta_id: i32, races: &[Race]) {
        self.races
            .insert_with_ttl(regatta_id, races.to_owned(), 1, TTL)
            .await;
        self.races.wait().await.unwrap();
    }

    pub(super) fn get_races(&self, regatta_id: i32) -> Option<Vec<Race>> {
        let opt_value_ref = self.races.get(&regatta_id);
        if let Some(value_ref) = opt_value_ref {
            let value = value_ref.value().clone();
            value_ref.release();
            debug!("Reading races of regatta {} from cache.", regatta_id);
            trace!("From cache: {:?}", value);
            return Some(value);
        }
        None
    }

    // race
    pub(super) async fn insert_race(&self, race: &Race) {
        self.race
            .insert_with_ttl(race.id, race.to_owned(), 1, TTL)
            .await;
        self.race.wait().await.unwrap();
    }

    pub(super) fn get_race(&self, race_id: i32) -> Option<Race> {
        let opt_value_ref = self.race.get(&race_id);
        if let Some(value_ref) = opt_value_ref {
            let value = value_ref.value().clone();
            value_ref.release();
            debug!("Reading race {} from cache.", race_id);
            trace!("From cache: {:?}", value);
            return Some(value);
        }
        None
    }

    // registrations
    pub(super) async fn insert_registrations(&self, race_id: i32, regs: &[Registration]) {
        self.regs
            .insert_with_ttl(race_id, regs.to_owned(), 1, TTL)
            .await;
        self.regs.wait().await.unwrap();
    }

    pub(super) fn get_registrations(&self, race_id: i32) -> Option<Vec<Registration>> {
        let opt_value_ref = self.regs.get(&race_id);
        // see also: https://doc.rust-lang.org/rust-by-example/flow_control/if_let.html
        if let Some(value_ref) = opt_value_ref {
            let value = value_ref.value().clone();
            value_ref.release();
            debug!("Reading registrations of race {} from cache.", race_id);
            trace!("From cache: {:?}", value);
            return Some(value);
        }
        None
    }
}
