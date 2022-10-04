use super::aquarius::{Heat, HeatRegistration, Regatta};
use log::debug;
use std::time::Duration;
use stretto::AsyncCache;

pub struct Cache {
    regatta_cache: AsyncCache<i32, Regatta>,
    heats_cache: AsyncCache<i32, Vec<Heat>>,
    heat_regs_cache: AsyncCache<i32, Vec<HeatRegistration>>,
}

impl Cache {
    /// Create a new `Cache`.
    pub fn new() -> Self {
        Cache {
            regatta_cache: AsyncCache::new(1 * 10, 1e6 as i64, async_std::task::spawn).unwrap(),
            heats_cache: AsyncCache::new(200 * 10, 1e6 as i64, async_std::task::spawn).unwrap(),
            heat_regs_cache: AsyncCache::new(200 * 10, 1e6 as i64, async_std::task::spawn).unwrap(),
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
            .insert_with_ttl(regatta.id, regatta.clone(), 1, Duration::from_secs(60))
            .await;
        self.regatta_cache.wait().await.unwrap();
    }

    pub async fn insert_heats(&self, regatta_id: i32, heats: &Vec<Heat>) {
        self.heats_cache
            .insert_with_ttl(regatta_id, heats.clone(), 1, Duration::from_secs(60))
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

    pub async fn insert_heat_regs(&self, heat_id: i32, heat_reg: &Vec<HeatRegistration>) {
        self.heat_regs_cache
            .insert_with_ttl(heat_id, heat_reg.clone(), 1, Duration::from_secs(60))
            .await;
        self.heat_regs_cache.wait().await.unwrap();
    }

    pub async fn get_heat_regs(&self, heat_reg_id: i32) -> Option<Vec<HeatRegistration>> {
        let opt_value_ref = self.heat_regs_cache.get(&heat_reg_id);
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
