use crate::aquarius::model::Regatta;
use crate::error::DbError;
use crate::tiberius::TiberiusPool;
use crate::timekeeper::Timestamp;
use crate::timekeeper::timestamp::Split;
use ::chrono::DateTime;
use ::chrono::Utc;
use ::std::collections::VecDeque;
use ::std::collections::vec_deque;
use ::std::sync::Arc;
use ::std::time::Instant;
use ::tracing::info;

/// A time strip is a collection of time stamps.
pub struct TimeStrip {
    // The ID of the regatta this time strip belongs to.
    regatta_id: i32,

    // A deque of time stamps.
    time_stamps: VecDeque<Timestamp>,

    pool: Arc<TiberiusPool>,
}

impl TimeStrip {
    pub async fn load(pool: Arc<TiberiusPool>) -> Result<Self, DbError> {
        let start = Instant::now();
        let pool_clone = pool.clone();
        let mut client = pool_clone.get().await?;
        let regatta = Regatta::query_active_regatta(&mut client).await?;
        let time_stamps = Timestamp::query_all_for_regatta(regatta.id, None, None, &mut client).await?;
        let time_strip = TimeStrip {
            regatta_id: regatta.id,
            time_stamps: VecDeque::from(time_stamps),
            pool,
        };
        info!(regatta_id = regatta.id, elapsed = ?start.elapsed(), "Loaded time strip:");
        Ok(time_strip)
    }

    pub async fn add_start(&mut self, time: Option<DateTime<Utc>>) -> Result<Timestamp, DbError> {
        let timestamp = Timestamp::from_time(time.unwrap_or_else(Utc::now), Split::Start);
        self.time_stamps.push_front(timestamp.clone());
        if let Some(timestamp) = self.time_stamps.front_mut() {
            let mut client = self.pool.get().await?;
            timestamp.persist(self.regatta_id, &mut client).await?;
            Ok(timestamp.clone())
        } else {
            Ok(timestamp)
        }
    }

    pub async fn add_finish(&mut self, time: Option<DateTime<Utc>>) -> Result<Timestamp, DbError> {
        let timestamp = Timestamp::from_time(time.unwrap_or_else(Utc::now), Split::Finish);
        self.time_stamps.push_front(timestamp.clone());
        if let Some(timestamp) = self.time_stamps.front_mut() {
            let mut client = self.pool.get().await?;
            timestamp.persist(self.regatta_id, &mut client).await?;
            Ok(timestamp.clone())
        } else {
            Ok(timestamp)
        }
    }

    pub async fn set_heat_nr(&mut self, timestamp: &Timestamp, heat_nr: i16) -> Result<Timestamp, DbError> {
        if let Some(timestamp) = self.time_stamps.iter_mut().find(|ts| ts.time == timestamp.time) {
            timestamp.set_heat_nr(heat_nr);
            let mut client = self.pool.get().await?;
            timestamp.update(&mut client).await?;
            return Ok(timestamp.clone());
        }
        Ok(timestamp.clone())
    }

    pub async fn set_bib(&mut self, timestamp: &Timestamp, bib: u8) -> Result<Timestamp, DbError> {
        if let Some(timestamp) = self.time_stamps.iter_mut().find(|ts| ts.time == timestamp.time) {
            timestamp.set_bib(bib);
            let mut client = self.pool.get().await?;
            timestamp.update(&mut client).await?;
            return Ok(timestamp.clone());
        }
        Ok(timestamp.clone())
    }

    pub async fn delete(&mut self, time: &DateTime<Utc>) -> Result<Timestamp, DbError> {
        if let Some(pos) = self.get_index(time)
            && let Some(timestamp) = self.time_stamps.remove(pos)
        {
            let mut client = self.pool.get().await?;
            timestamp.delete(&mut client).await?;
            return Ok(timestamp);
        }
        Err(DbError::Custom("Timestamp not found".to_string()))
    }

    /// Returns an iterator over the time stamps.
    pub fn iter(&self) -> vec_deque::Iter<'_, Timestamp> {
        self.time_stamps.iter()
    }

    /// Returns the number of time stamps.
    pub fn len(&self) -> usize {
        self.time_stamps.len()
    }

    /// Returns `true` if there are no time stamps.
    pub fn is_empty(&self) -> bool {
        self.time_stamps.is_empty()
    }

    /// Returns a reference to the time stamp at the given index.
    pub fn get(&self, index: usize) -> Option<&Timestamp> {
        self.time_stamps.get(index)
    }

    /// Returns a reference to the time stamp with the given time.
    pub fn get_by_time(&self, time: &DateTime<Utc>) -> Option<&Timestamp> {
        self.time_stamps.iter().find(|timestamp| timestamp.time == *time)
    }

    /// Returns a `Vec` containing clones of all time stamps.
    pub fn to_vec(&self) -> Vec<Timestamp> {
        self.time_stamps.clone().into()
    }

    fn get_index(&self, time: &DateTime<Utc>) -> Option<usize> {
        self.time_stamps.iter().position(|timestamp| timestamp.time == *time)
    }
}
