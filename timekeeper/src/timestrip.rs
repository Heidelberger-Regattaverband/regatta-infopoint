use chrono::{DateTime, Utc};
use log::info;
use std::time::SystemTime;

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
    /// The time of the event.
    pub(crate) time: DateTime<Utc>,

    /// The type of the time stamp.
    pub(crate) stamp_type: TimeStampType,
}

impl TimeStamp {
    /// Create a new time stamp with the current time in UTC.
    ///
    /// # Arguments
    /// * `stamp_type` - The type of the time stamp.
    fn now(stamp_type: TimeStampType) -> TimeStamp {
        let now = SystemTime::now();
        let time = DateTime::from(now);
        TimeStamp { time, stamp_type }
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
