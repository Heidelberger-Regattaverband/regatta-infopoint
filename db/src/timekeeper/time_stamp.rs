use chrono::{DateTime, Local};
use std::sync::atomic::{AtomicU64, Ordering};
use strum_macros::Display;

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
    pub time: DateTime<Local>,

    /// The type of the time stamp.
    pub stamp_type: TimeStampType,

    /// The heat number.
    pub heat_nr: Option<u16>,

    /// The bib number.
    pub bib: Option<u32>,
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
            time: Local::now(),
            stamp_type,
            heat_nr: None,
            bib: None,
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
