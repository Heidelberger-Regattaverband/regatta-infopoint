use crate::tiberius::TiberiusPool;
use crate::{
    aquarius::model::Regatta,
    error::DbError,
    timekeeper::time_stamp::{Split, TimeStamp},
};
use ::std::collections::VecDeque;
use ::std::sync::Arc;
use ::std::time::Instant;
use ::tracing::info;

/// A time strip is a collection of time stamps.
pub struct TimeStrip {
    // The ID of the regatta this time strip belongs to.
    regatta_id: i32,

    // A deque of time stamps.
    pub time_stamps: VecDeque<TimeStamp>,

    pool: Arc<TiberiusPool>,
}

impl TimeStrip {
    pub async fn load(pool: Arc<TiberiusPool>) -> Result<Self, DbError> {
        let start = Instant::now();
        let pool_clone = pool.clone();
        let mut client = pool_clone.get().await?;
        let regatta = Regatta::query_active_regatta(&mut client).await?;
        let time_stamps = TimeStamp::query_all_for_regatta(regatta.id, None, None, &mut client).await?;
        let time_strip = TimeStrip {
            regatta_id: regatta.id,
            time_stamps: VecDeque::from(time_stamps),
            pool,
        };
        info!(regatta_id = regatta.id, elapsed = ?start.elapsed(), "Loaded time strip:");
        Ok(time_strip)
    }

    pub async fn add_start(&mut self) -> Result<(), DbError> {
        let time_stamp = TimeStamp::now(Split::Start);
        info!(?time_stamp, "Start time stamp:");
        self.time_stamps.push_front(time_stamp);
        if let Some(ts) = self.time_stamps.front_mut() {
            let mut client = self.pool.get().await?;
            ts.persist(self.regatta_id, &mut client).await?;
        }
        Ok(())
    }

    pub async fn add_finish(&mut self) -> Result<(), DbError> {
        let time_stamp = TimeStamp::now(Split::Finish);
        info!(?time_stamp, "Finish time stamp:");
        self.time_stamps.push_front(time_stamp);
        if let Some(ts) = self.time_stamps.front_mut() {
            let mut client = self.pool.get().await?;
            ts.persist(self.regatta_id, &mut client).await?;
        }
        Ok(())
    }

    pub async fn set_heat_nr(&mut self, time_stamp: &TimeStamp, heat_nr: i16) -> Result<TimeStamp, DbError> {
        if let Some(time_stamp) = self.time_stamps.iter_mut().find(|ts| ts.time == time_stamp.time) {
            time_stamp.set_heat_nr(heat_nr);
            let mut client = self.pool.get().await?;
            time_stamp.update(&mut client).await?;
            return Ok(time_stamp.clone());
        }
        Ok(time_stamp.clone())
    }

    pub async fn set_bib(&mut self, time_stamp: &TimeStamp, bib: u8) -> Result<TimeStamp, DbError> {
        if let Some(time_stamp) = self.time_stamps.iter_mut().find(|ts| ts.time == time_stamp.time) {
            time_stamp.set_bib(bib);
            let mut client = self.pool.get().await?;
            time_stamp.update(&mut client).await?;
            return Ok(time_stamp.clone());
        }
        Ok(time_stamp.clone())
    }

    pub async fn delete(&mut self, time_stamp: &TimeStamp) -> Result<(), DbError> {
        if let Some(pos) = self.get_index(time_stamp)
            && let Some(time_stamp) = self.time_stamps.remove(pos)
        {
            let mut client = self.pool.get().await?;
            time_stamp.delete(&mut client).await?;
        }
        Ok(())
    }

    fn get_index(&self, time_stamp: &TimeStamp) -> Option<usize> {
        self.time_stamps.iter().position(|ts| ts.time == time_stamp.time)
    }
}
