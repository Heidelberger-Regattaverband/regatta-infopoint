use super::model::{Heat, HeatRegistration, Race, Regatta, Registration, Score};
use log::{debug, trace};
use std::time::Duration;
use stretto::AsyncCache;

pub(super) struct Cache {
    regatta_cache: AsyncCache<i32, Regatta>,
    races_cache: AsyncCache<i32, Vec<Race>>,
    race_cache: AsyncCache<i32, Race>,
    regs_cache: AsyncCache<i32, Vec<Registration>>,
    heats_cache: AsyncCache<i32, Vec<Heat>>,
    heat_regs_cache: AsyncCache<i32, Vec<HeatRegistration>>,
    scores_cache: AsyncCache<i32, Vec<Score>>,
}

const TTL: Duration = Duration::from_secs(120);

impl Cache {
    /// Creates a new `Cache`.
    pub(super) fn new() -> Self {
        const MAX_COST: i64 = 1e6 as i64;
        const MAX_RAGATTA_COUNT: usize = 5;
        const MAX_RACES_COUNT: usize = 200;
        const MAX_HEATS_COUNT: usize = 350;

        Cache {
            regatta_cache: AsyncCache::new(MAX_RAGATTA_COUNT, MAX_COST, async_std::task::spawn)
                .unwrap(),
            races_cache: AsyncCache::new(MAX_RAGATTA_COUNT, MAX_COST, async_std::task::spawn)
                .unwrap(),
            race_cache: AsyncCache::new(MAX_RACES_COUNT, MAX_COST, async_std::task::spawn).unwrap(),
            regs_cache: AsyncCache::new(MAX_RACES_COUNT, MAX_COST, async_std::task::spawn).unwrap(),
            heats_cache: AsyncCache::new(MAX_RAGATTA_COUNT, MAX_COST, async_std::task::spawn)
                .unwrap(),
            heat_regs_cache: AsyncCache::new(MAX_HEATS_COUNT, MAX_COST, async_std::task::spawn)
                .unwrap(),
            scores_cache: AsyncCache::new(MAX_RAGATTA_COUNT, MAX_COST, async_std::task::spawn)
                .unwrap(),
        }
    }

    // regattas
    pub(super) fn get_regatta(&self, regatta_id: i32) -> Option<Regatta> {
        let opt_value_ref = self.regatta_cache.get(&regatta_id);
        // see also: https://doc.rust-lang.org/rust-by-example/flow_control/if_let.html
        if let Some(value_ref) = opt_value_ref {
            let value = value_ref.value().clone();
            value_ref.release();
            debug!("Reading regatta {} from cache.", regatta_id);
            trace!("From cache: {:?}", value);
            return Some(value);
        }
        None
    }

    pub(super) async fn insert_regatta(&self, regatta: &Regatta) {
        self.regatta_cache
            .insert_with_ttl(regatta.id, regatta.clone(), 1, TTL)
            .await;
        self.regatta_cache.wait().await.unwrap();
    }

    // heats

    pub(super) async fn insert_heats(&self, regatta_id: i32, heats: &[Heat]) {
        self.heats_cache
            .insert_with_ttl(regatta_id, heats.to_owned(), 1, TTL)
            .await;
        self.heats_cache.wait().await.unwrap();
    }

    pub(super) fn get_heats(&self, regatta_id: i32) -> Option<Vec<Heat>> {
        let opt_value_ref = self.heats_cache.get(&regatta_id);
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
        self.heat_regs_cache
            .insert_with_ttl(heat_id, heat_regs.to_owned(), 1, TTL)
            .await;
        self.heat_regs_cache.wait().await.unwrap();
    }

    pub fn get_heat_regs(&self, heat_id: i32) -> Option<Vec<HeatRegistration>> {
        let opt_value_ref = self.heat_regs_cache.get(&heat_id);
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
        self.scores_cache
            .insert_with_ttl(regatta_id, scores.to_owned(), 1, TTL)
            .await;
        self.scores_cache.wait().await.unwrap();
    }

    pub(super) fn get_scores(&self, regatta_id: i32) -> Option<Vec<Score>> {
        let opt_value_ref = self.scores_cache.get(&regatta_id);
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
        self.races_cache
            .insert_with_ttl(regatta_id, races.to_owned(), 1, TTL)
            .await;
        self.races_cache.wait().await.unwrap();
    }

    pub(super) fn get_races(&self, regatta_id: i32) -> Option<Vec<Race>> {
        let opt_value_ref = self.races_cache.get(&regatta_id);
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
        self.race_cache
            .insert_with_ttl(race.id, race.to_owned(), 1, TTL)
            .await;
        self.race_cache.wait().await.unwrap();
    }

    pub(super) fn get_race(&self, race_id: i32) -> Option<Race> {
        let opt_value_ref = self.race_cache.get(&race_id);
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
        self.regs_cache
            .insert_with_ttl(race_id, regs.to_owned(), 1, TTL)
            .await;
        self.regs_cache.wait().await.unwrap();
    }

    pub(super) fn get_registrations(&self, race_id: i32) -> Option<Vec<Registration>> {
        let opt_value_ref = self.regs_cache.get(&race_id);
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
