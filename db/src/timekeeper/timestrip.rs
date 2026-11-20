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

/// An in-memory snapshot of start and finish timestamps for a single regatta.
/// Loading retrieves the 30 most recent persisted timestamps, newest first.
/// Newly added timestamps are inserted at the front without re-sorting, even
/// when their supplied times are older than existing entries. Changes made by
/// other clients are not automatically reflected in this snapshot.
/// # Persistence failures
/// Mutating methods change the local snapshot before acquiring a connection or
/// executing SQL. An error therefore does not leave the snapshot unchanged:
/// failed additions or assignments remain locally, and failed deletions have
/// already removed their entries. Reload the strip to reconcile it with the
/// database after a persistence failure.
pub struct TimeStrip {
    /// The ID of the regatta this time strip belongs to.
    regatta_id: i32,

    /// The loaded timestamps, with subsequent additions inserted at the front.
    time_stamps: VecDeque<Timestamp>,

    /// The shared connection pool used for loading and persisting timestamps.
    pool: Arc<TiberiusPool>,
}

impl TimeStrip {
    /// Loads timestamps for the explicitly selected regatta and uses it for new timestamps.
    /// # Arguments
    /// * `regatta_id` - The selected regatta ID; the active regatta is not resolved here.
    /// * `pool` - The shared database connection pool retained for subsequent writes.
    /// # Returns
    /// A strip containing up to 30 timestamps in descending timestamp order.
    /// Returns an empty strip if no matching timestamps exist; this does not
    /// verify that the regatta itself exists.
    /// # Errors
    /// Returns a [`DbError`] if connection acquisition or the timestamp query fails.
    /// # Panics
    /// Panics if row decoding encounters a missing, NULL, or incorrectly typed
    /// required column.
    pub async fn load(regatta_id: i32, pool: Arc<TiberiusPool>) -> Result<Self, DbError> {
        let start = Instant::now();
        let time_stamps = {
            let mut client = pool.get().await?;
            Timestamp::query_all_for_regatta(regatta_id, None, None, &mut client).await?
        };
        let time_strip = TimeStrip {
            regatta_id,
            time_stamps: VecDeque::from(time_stamps),
            pool,
        };
        info!(regatta_id, elapsed = ?start.elapsed(), "Loaded time strip:");
        Ok(time_strip)
    }

    /// Adds a start timestamp at the front of the strip and persists it.
    /// # Arguments
    /// * `time` - The UTC start time, or `None` to sample [`Utc::now`] when processed.
    /// # Returns
    /// A clone of the newly added, persisted [`Timestamp`] with [`Split::Start`].
    /// # Errors
    /// Returns a [`DbError`] if connection acquisition or insertion fails. The
    /// new timestamp remains in the local snapshot; see [`TimeStrip`]'s
    /// persistence-failure notes.
    pub async fn add_start(&mut self, time: Option<DateTime<Utc>>) -> Result<Timestamp, DbError> {
        self.add_timestamp(Split::Start, time).await
    }

    /// Adds a finish timestamp at the front of the strip and persists it.
    /// # Arguments
    /// * `time` - The UTC finish time, or `None` to sample [`Utc::now`] when processed.
    /// # Returns
    /// A clone of the newly added, persisted [`Timestamp`] with [`Split::Finish`].
    /// # Errors
    /// Returns a [`DbError`] if connection acquisition or insertion fails. The
    /// new timestamp remains in the local snapshot; see [`TimeStrip`]'s
    /// persistence-failure notes.
    pub async fn add_finish(&mut self, time: Option<DateTime<Utc>>) -> Result<Timestamp, DbError> {
        self.add_timestamp(Split::Finish, time).await
    }

    /// Creates a timestamp, inserts it at the front, and persists it for this regatta.
    /// # Arguments
    /// * `split` - Whether the timestamp records a start or finish.
    /// * `time` - The UTC event time, or `None` to use [`Utc::now`].
    /// # Returns
    /// A clone of the newly persisted timestamp.
    /// # Errors
    /// Returns a [`DbError`] if connection acquisition or insertion fails after
    /// the timestamp has been added to the local snapshot.
    async fn add_timestamp(&mut self, split: Split, time: Option<DateTime<Utc>>) -> Result<Timestamp, DbError> {
        let timestamp = Timestamp::from_time(time.unwrap_or_else(Utc::now), split);
        self.time_stamps.push_front(timestamp.clone());
        if let Some(timestamp) = self.time_stamps.front_mut() {
            let mut client = self.pool.get().await?;
            timestamp.persist(self.regatta_id, &mut client).await?;
            Ok(timestamp.clone())
        } else {
            Ok(timestamp)
        }
    }

    /// Assigns a heat number to a local timestamp and persists its assignments.
    /// The timestamp is identified by its time, not object identity. The database
    /// update writes both the stored heat number and bib; this method does not
    /// validate whether the selected heat is open or belongs to this regatta.
    /// # Arguments
    /// * `timestamp` - The timestamp whose time identifies the first local match.
    /// * `heat_nr` - The heat number to assign.
    /// # Returns
    /// A clone of the updated, persisted timestamp.
    /// # Errors
    /// Returns [`DbError::Custom`] if no local timestamp matches, or
    /// [`DbError::NotFound`] if no database row is updated. Connection and SQL
    /// errors are propagated. Persistence errors leave the local assignment
    /// changed; see [`TimeStrip`]'s persistence-failure notes.
    pub async fn set_heat_nr(&mut self, timestamp: &Timestamp, heat_nr: i16) -> Result<Timestamp, DbError> {
        if let Some(timestamp) = self.time_stamps.iter_mut().find(|ts| ts.time == timestamp.time) {
            timestamp.set_heat_nr(heat_nr);
            let mut client = self.pool.get().await?;
            timestamp.update(&mut client).await?;
            return Ok(timestamp.clone());
        }
        Err(DbError::Custom(format!("Timestamp not found: {:?}", timestamp.time)))
    }

    /// Assigns a bib to a local timestamp and persists its assignments.
    /// The timestamp is identified by its time, not object identity. The database
    /// update writes both the stored heat number and bib; this method does not
    /// validate whether the bib belongs to the assigned heat.
    /// # Arguments
    /// * `timestamp` - The timestamp whose time identifies the first local match.
    /// * `bib` - The bib number to assign.
    /// # Returns
    /// A clone of the updated, persisted timestamp.
    /// # Errors
    /// Returns [`DbError::Custom`] if no local timestamp matches, or
    /// [`DbError::NotFound`] if no database row is updated. Connection and SQL
    /// errors are propagated. Persistence errors leave the local assignment
    /// changed; see [`TimeStrip`]'s persistence-failure notes.
    pub async fn set_bib(&mut self, timestamp: &Timestamp, bib: u8) -> Result<Timestamp, DbError> {
        if let Some(timestamp) = self.time_stamps.iter_mut().find(|ts| ts.time == timestamp.time) {
            timestamp.set_bib(bib);
            let mut client = self.pool.get().await?;
            timestamp.update(&mut client).await?;
            return Ok(timestamp.clone());
        }
        Err(DbError::Custom(format!("Timestamp not found: {:?}", timestamp.time)))
    }

    /// Removes the first local timestamp with the given time and deletes it from the database.
    /// The database deletion matches the timestamp value alone.
    /// # Arguments
    /// * `time` - The exact UTC time used to identify the timestamp.
    /// # Returns
    /// The timestamp removed from the local snapshot.
    /// # Errors
    /// Returns [`DbError::Custom`] if no local timestamp matches, or
    /// [`DbError::NotFound`] if no database row is deleted. Connection and SQL
    /// errors are propagated. Persistence errors occur after local removal;
    /// see [`TimeStrip`]'s persistence-failure notes.
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

    /// Returns a borrowed iterator over the timestamps in stored order.
    /// # Returns
    /// An iterator from the front to the back of the local snapshot, without
    /// querying the database.
    pub fn iter(&self) -> vec_deque::Iter<'_, Timestamp> {
        self.time_stamps.iter()
    }

    /// Returns the number of timestamps in the local snapshot.
    /// This is not the total number of persisted timestamps for the regatta.
    pub fn len(&self) -> usize {
        self.time_stamps.len()
    }

    /// Returns `true` if the local snapshot contains no timestamps.
    pub fn is_empty(&self) -> bool {
        self.time_stamps.is_empty()
    }

    /// Returns a borrowed timestamp at a zero-based index in stored order.
    /// # Arguments
    /// * `index` - The index in the local snapshot; zero identifies the front entry.
    /// # Returns
    /// The timestamp at `index`, or `None` if the index is out of bounds.
    pub fn get(&self, index: usize) -> Option<&Timestamp> {
        self.time_stamps.get(index)
    }

    /// Finds the first local timestamp with exactly the given time.
    /// # Arguments
    /// * `time` - The UTC time to match.
    /// # Returns
    /// A borrowed matching timestamp, or `None` if no local entry matches.
    pub fn get_by_time(&self, time: &DateTime<Utc>) -> Option<&Timestamp> {
        self.time_stamps.iter().find(|timestamp| timestamp.time == *time)
    }

    /// Returns an owned vector of timestamp clones in stored order.
    /// Changing the returned vector does not change this strip. No database
    /// query is performed.
    pub fn to_vec(&self) -> Vec<Timestamp> {
        self.time_stamps.clone().into()
    }

    /// Finds the index of the first local timestamp with exactly the given time.
    /// # Arguments
    /// * `time` - The UTC time to match.
    /// # Returns
    /// The zero-based index in stored order, or `None` if no local entry matches.
    fn get_index(&self, time: &DateTime<Utc>) -> Option<usize> {
        self.time_stamps.iter().position(|timestamp| timestamp.time == *time)
    }
}
