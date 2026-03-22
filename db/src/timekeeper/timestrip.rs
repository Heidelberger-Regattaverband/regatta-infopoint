use crate::tiberius::TiberiusClient;
use crate::{
    aquarius::model::Regatta,
    error::DbError,
    timekeeper::time_stamp::{Split, TimeStamp},
};
use ::std::time::Instant;
use ::tracing::info;

/// A time strip is a collection of time stamps.
pub struct TimeStrip {
    // The ID of the regatta this time strip belongs to.
    regatta_id: i32,

    // A vector of time stamps.
    pub time_stamps: Vec<TimeStamp>,
}

impl TimeStrip {
    pub async fn load(client: &mut TiberiusClient) -> Result<Self, DbError> {
        let start = Instant::now();
        let regatta = Regatta::query_active_regatta(client).await?;
        let time_stamps = TimeStamp::query_all_for_regatta(regatta.id, None, None, client).await?;
        let time_strip = TimeStrip {
            regatta_id: regatta.id,
            time_stamps,
        };
        info!(regatta_id = regatta.id, elapsed = ?start.elapsed(), "Loaded time strip:");
        Ok(time_strip)
    }

    pub async fn add_start(&mut self, client: &mut TiberiusClient) -> Result<(), DbError> {
        let time_stamp = TimeStamp::now(Split::Start);
        info!(?time_stamp, "Start time stamp:");
        self.time_stamps.push(time_stamp);
        if let Some(ts) = self.time_stamps.last_mut() {
            ts.persist(self.regatta_id, client).await?;
        }
        Ok(())
    }

    pub async fn add_finish(&mut self, client: &mut TiberiusClient) -> Result<(), DbError> {
        let time_stamp = TimeStamp::now(Split::Finish);
        info!(?time_stamp, "Finish time stamp:");
        self.time_stamps.push(time_stamp);
        if let Some(ts) = self.time_stamps.last_mut() {
            ts.persist(self.regatta_id, client).await?;
        }
        Ok(())
    }

    pub async fn set_heat_nr(
        &mut self,
        time_stamp: &TimeStamp,
        heat_nr: i16,
        client: &mut TiberiusClient,
    ) -> Result<TimeStamp, DbError> {
        if let Some(time_stamp) = self.time_stamps.iter_mut().find(|ts| ts.time == time_stamp.time) {
            time_stamp.set_heat_nr(heat_nr);
            time_stamp.update(client).await?;
            return Ok(time_stamp.clone());
        }
        Ok(time_stamp.clone())
    }

    pub async fn set_bib(
        &mut self,
        time_stamp: &TimeStamp,
        bib: u8,
        client: &mut TiberiusClient,
    ) -> Result<TimeStamp, DbError> {
        if let Some(time_stamp) = self.time_stamps.iter_mut().find(|ts| ts.time == time_stamp.time) {
            time_stamp.set_bib(bib);
            time_stamp.update(client).await?;
            return Ok(time_stamp.clone());
        }
        Ok(time_stamp.clone())
    }

    pub async fn delete(&mut self, time_stamp: &TimeStamp, client: &mut TiberiusClient) -> Result<(), DbError> {
        if let Some(pos) = self.get_index(time_stamp) {
            let time_stamp = self.time_stamps.remove(pos);
            time_stamp.delete(client).await?;
        }
        Ok(())
    }

    fn get_index(&self, time_stamp: &TimeStamp) -> Option<usize> {
        self.time_stamps.iter().position(|ts| ts.time == time_stamp.time)
    }
}
