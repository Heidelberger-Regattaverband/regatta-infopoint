use crate::{
    aquarius::model::Regatta,
    tiberius::TiberiusPool,
    timekeeper::time_stamp::{Split, TimeStamp},
};
use log::info;
use tiberius::error::Error as DbError;

/// A time strip is a collection of time stamps.
pub struct TimeStrip {
    regatta_id: i32,
    pool: &'static TiberiusPool,
    // A vector of time stamps.
    pub time_stamps: Vec<TimeStamp>,
}

impl TimeStrip {
    pub async fn load(pool: &'static TiberiusPool) -> Result<Self, DbError> {
        let regatta = Regatta::query_active_regatta(pool).await?;
        info!("Loading time strip for regatta ID: {0}", regatta.id);
        let time_stamps = TimeStamp::query_all_for_regatta(regatta.id, pool).await?;
        Ok(TimeStrip {
            regatta_id: regatta.id,
            time_stamps,
            pool,
        })
    }

    pub fn add_new_start(&mut self) {
        let mut time_stamp = TimeStamp::now(Split::Start);
        info!("Start time stamp: {time_stamp:?}");
        self.time_stamps.push(time_stamp.clone());
        let regatta_id = self.regatta_id;
        let pool = self.pool;
        tokio::spawn(async move {
            time_stamp.persist(regatta_id, pool).await.unwrap();
            time_stamp.persisted = true;
        });
    }

    pub fn add_new_finish(&mut self) {
        let mut time_stamp = TimeStamp::now(Split::Finish);
        info!("Finish time stamp: {time_stamp:?}");
        self.time_stamps.push(time_stamp.clone());
        let regatta_id = self.regatta_id;
        let pool = self.pool;
        tokio::spawn(async move {
            time_stamp.persist(regatta_id, pool).await.unwrap();
            time_stamp.persisted = true;
        });
    }

    pub fn assign_heat_nr(&mut self, time_stamp: &TimeStamp, heat_nr: i16) -> Option<TimeStamp> {
        if let Some(time_stamp) = self.time_stamps.iter_mut().find(|ts| ts.index == time_stamp.index) {
            time_stamp.heat_nr = Some(heat_nr);
            return Some(time_stamp.clone());
        }
        None
    }

    pub fn delete(&mut self, time_stamp: &TimeStamp) {
        if let Some(pos) = self.get_index(time_stamp) {
            let time_stamp = self.time_stamps.remove(pos);
            let pool = self.pool;
            tokio::spawn(async move {
                time_stamp.delete(pool).await.unwrap();
            });
        }
    }

    fn get_index(&self, time_stamp: &TimeStamp) -> Option<usize> {
        self.time_stamps.iter().position(|ts| ts.time == time_stamp.time)
    }
}
