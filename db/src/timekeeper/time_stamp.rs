use crate::aquarius::model::utils;
use crate::tiberius::{RowColumn, TiberiusPool, TryRowColumn};
use chrono::Utc;
use chrono::{DateTime, Local};
use std::sync::atomic::{AtomicU64, Ordering};
use strum_macros::Display;
use tiberius::{Query, Row, error::Error as DbError};

static TIME_STAMP_INDEX: AtomicU64 = AtomicU64::new(0);

fn next_index() -> u64 {
    TIME_STAMP_INDEX.fetch_add(1, Ordering::SeqCst)
}

/// A time stamp of an event, such as a start or finish time stamp in a race.
#[derive(Debug, Clone)]
pub struct TimeStamp {
    /// The index of the time stamp.
    pub index: u64,

    /// The time of the event.
    pub time: DateTime<Utc>,

    /// The type of the time stamp.
    pub stamp_type: TimeStampType,

    /// The heat number.
    pub heat_nr: Option<i16>,

    /// The bib number.
    pub bib: Option<u8>,
}

impl TimeStamp {
    /// Create a new time stamp with the current time in UTC.
    ///
    /// # Arguments
    /// * `stamp_type` - The type of the time stamp.
    /// # Returns
    /// A new time stamp with the current time.
    pub(crate) fn now(stamp_type: TimeStampType) -> TimeStamp {
        TimeStamp {
            index: next_index(),
            time: Local::now().to_utc(),
            stamp_type,
            heat_nr: None,
            bib: None,
        }
    }

    pub async fn query_for_regatta(regatta_id: i32, pool: &TiberiusPool) -> Result<Vec<TimeStamp>, DbError> {
        let mut query = Query::new(format!(
            "SELECT timestamp, event_id, type, heat_nr, bib
             FROM HRV_Timestrip WHERE event_id = @P1 ORDER BY timestamp DESC",
        ));
        query.bind(regatta_id);

        let mut client = pool.get().await;
        let stream = query.query(&mut client).await?;
        let time_stamps = utils::get_rows(stream).await?;
        Ok(time_stamps.into_iter().map(|row| TimeStamp::from(&row)).collect())
    }
}

impl From<&Row> for TimeStamp {
    fn from(row: &Row) -> Self {
        TimeStamp {
            index: next_index(),
            time: row.get_column("timestamp"),
            stamp_type: TimeStampType::Start,
            heat_nr: row.try_get_column("heat_nr"),
            bib: row.try_get_column("bib"),
        }
    }
}

/// The type of a time stamp.
#[derive(Debug, Clone, Display, Copy)]
pub enum TimeStampType {
    /// A start time stamp.
    #[strum(to_string = "Start")]
    Start,

    /// A finish time stamp.
    #[strum(to_string = "Ziel")]
    Finish,
}
