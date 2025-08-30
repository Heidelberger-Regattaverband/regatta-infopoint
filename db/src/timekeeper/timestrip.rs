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

    pub async fn add_start(&mut self) -> Result<(), DbError> {
        let time_stamp = TimeStamp::now(Split::Start);
        info!("Start time stamp: {time_stamp:?}");
        self.time_stamps.push(time_stamp);
        if let Some(ts) = self.time_stamps.last_mut() {
            ts.persist(self.regatta_id, self.pool).await?;
        }
        Ok(())
    }

    pub async fn add_finish(&mut self) -> Result<(), DbError> {
        let time_stamp = TimeStamp::now(Split::Finish);
        info!("Finish time stamp: {time_stamp:?}");
        self.time_stamps.push(time_stamp);
        if let Some(ts) = self.time_stamps.last_mut() {
            ts.persist(self.regatta_id, self.pool).await?;
        }
        Ok(())
    }

    pub async fn set_heat_nr(&mut self, time_stamp: &TimeStamp, heat_nr: i16) -> Result<TimeStamp, DbError> {
        if let Some(time_stamp) = self.time_stamps.iter_mut().find(|ts| ts.time == time_stamp.time) {
            time_stamp.set_heat_nr(heat_nr);
            time_stamp.update(self.pool).await?;
            return Ok(time_stamp.clone());
        }
        Ok(time_stamp.clone())
    }

    pub async fn set_bib(&mut self, time_stamp: &TimeStamp, bib: u8) -> Result<TimeStamp, DbError> {
        if let Some(time_stamp) = self.time_stamps.iter_mut().find(|ts| ts.time == time_stamp.time) {
            time_stamp.set_bib(bib);
            time_stamp.update(self.pool).await?;
            return Ok(time_stamp.clone());
        }
        Ok(time_stamp.clone())
    }

    pub async fn delete(&mut self, time_stamp: &TimeStamp) -> Result<(), DbError> {
        if let Some(pos) = self.get_index(time_stamp) {
            let time_stamp = self.time_stamps.remove(pos);
            let pool = self.pool;
            time_stamp.delete(pool).await?;
        }
        Ok(())
    }

    fn get_index(&self, time_stamp: &TimeStamp) -> Option<usize> {
        self.time_stamps.iter().position(|ts| ts.time == time_stamp.time)
    }
}
