use chrono::{DateTime, Local};
use log::info;
use std::sync::atomic::{AtomicU64, Ordering};

static TIME_STAMP_INDEX: AtomicU64 = AtomicU64::new(0);

fn next_index() -> u64 {
    TIME_STAMP_INDEX.fetch_add(1, Ordering::SeqCst); // Automatically handles wrapping at 256!
    TIME_STAMP_INDEX.load(Ordering::SeqCst)
}

/// A time strip is a collection of time stamps.
#[derive(Default)]
pub(crate) struct TimeStrip {
    // A vector of time stamps.
    pub(crate) time_stamps: Vec<TimeStamp>,
}

impl TimeStrip {
    pub(crate) fn add_new_start(&mut self) {
        let time_stamp = TimeStamp::now(TimeStampType::Start);
        info!("Start time stamp: {:?}", time_stamp);
        self.time_stamps.push(time_stamp);
    }

    pub(crate) fn add_new_finish(&mut self) {
        let time_stamp = TimeStamp::now(TimeStampType::Finish);
        info!("Finish time stamp: {:?}", time_stamp);
        self.time_stamps.push(time_stamp);
    }
}

/// A time stamp of an event.
#[derive(Debug, Clone)]
pub(crate) struct TimeStamp {
    /// The index of the time stamp.
    pub(crate) index: u64,

    /// The time of the event.
    pub(crate) time: DateTime<Local>,

    /// The type of the time stamp.
    pub(crate) stamp_type: TimeStampType,
}

impl TimeStamp {
    /// Create a new time stamp with the current time in UTC.
    ///
    /// # Arguments
    /// * `stamp_type` - The type of the time stamp.
    fn now(stamp_type: TimeStampType) -> TimeStamp {
        TimeStamp {
            index: next_index(),
            time: Local::now(),
            stamp_type,
        }
    }
}

/// The type of a time stamp.
#[derive(Debug, Clone)]
pub(crate) enum TimeStampType {
    /// A start time stamp.
    Start,

    /// A finish time stamp.
    Finish,
}
