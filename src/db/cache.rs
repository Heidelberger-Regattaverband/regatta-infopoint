use super::aquarius::{Heat, HeatRegistration, Regatta, Score};
use log::debug;
use std::time::Duration;
use stretto::AsyncCache;

pub struct Cache {
    regatta_cache: AsyncCache<i32, Regatta>,
    heats_cache: AsyncCache<i32, Vec<Heat>>,
    heat_regs_cache: AsyncCache<i32, Vec<HeatRegistration>>,
    scores_cache: AsyncCache<i32, Vec<Score>>,
}

const TTL: Duration = Duration::from_secs(60);

impl Cache {
    /// Creates a new `Cache`.
    pub fn new() -> Self {
        Cache {
            regatta_cache: AsyncCache::new(10, 1e6 as i64, async_std::task::spawn).unwrap(),
            heats_cache: AsyncCache::new(200 * 10, 1e6 as i64, async_std::task::spawn).unwrap(),
            heat_regs_cache: AsyncCache::new(200 * 10, 1e6 as i64, async_std::task::spawn).unwrap(),
            scores_cache: AsyncCache::new(10, 1e6 as i64, async_std::task::spawn).unwrap(),
        }
    }

    pub async fn get_regatta(&self, regatta_id: i32) -> Option<Regatta> {
        let opt_value_ref = self.regatta_cache.get(&regatta_id);
        if opt_value_ref.is_some() {
            let value_ref = opt_value_ref.unwrap();
            let value = value_ref.value().clone();
            value_ref.release();
            debug!("From cache: {:?}", value);
            return Some(value);
        }
        None
    }

    pub async fn insert_regatta(&self, regatta: &Regatta) {
        self.regatta_cache
            .insert_with_ttl(regatta.id, regatta.clone(), 1, TTL)
            .await;
        self.regatta_cache.wait().await.unwrap();
    }

    // heats

    pub async fn insert_heats(&self, regatta_id: i32, heats: &[Heat]) {
        self.heats_cache
            .insert_with_ttl(regatta_id, heats.to_owned().clone(), 1, TTL)
            .await;
        self.heats_cache.wait().await.unwrap();
    }

    pub async fn get_heats(&self, regatta_id: i32) -> Option<Vec<Heat>> {
        let opt_value_ref = self.heats_cache.get(&regatta_id);
        if opt_value_ref.is_some() {
            let value_ref = opt_value_ref.unwrap();
            let value = value_ref.value().clone();
            value_ref.release();
            debug!("From cache: {:?}", value);
            return Some(value);
        }
        None
    }

    // heat_registrations

    pub async fn insert_heat_regs(&self, heat_id: i32, heat_reg: &[HeatRegistration]) {
        self.heat_regs_cache
            .insert_with_ttl(heat_id, heat_reg.to_owned().clone(), 1, TTL)
            .await;
        self.heat_regs_cache.wait().await.unwrap();
    }

    pub async fn get_heat_regs(&self, heat_id: i32) -> Option<Vec<HeatRegistration>> {
        let opt_value_ref = self.heat_regs_cache.get(&heat_id);
        if opt_value_ref.is_some() {
            let value_ref = opt_value_ref.unwrap();
            let value = value_ref.value().clone();
            value_ref.release();
            debug!("From cache: {:?}", value);
            return Some(value);
        }
        None
    }

    // scores

    pub async fn insert_scores(&self, regatta_id: i32, scores: &[Score]) {
        self.scores_cache
            .insert_with_ttl(
                regatta_id,
                scores.to_owned().clone(),
                1,
                Duration::from_secs(60),
            )
            .await;
        self.heat_regs_cache.wait().await.unwrap();
    }

    pub async fn get_scores(&self, regatta_id: i32) -> Option<Vec<Score>> {
        let opt_value_ref = self.scores_cache.get(&regatta_id);
        if opt_value_ref.is_some() {
            let value_ref = opt_value_ref.unwrap();
            let value = value_ref.value().clone();
            value_ref.release();
            debug!("From cache: {:?}", value);
            return Some(value);
        }
        None
    }
}
