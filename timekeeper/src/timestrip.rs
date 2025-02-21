use chrono::{DateTime, Local};
use log::info;
use std::sync::atomic::{AtomicU64, Ordering};
use strum_macros::Display;

static TIME_STAMP_INDEX: AtomicU64 = AtomicU64::new(0);

fn next_index() -> u64 {
    TIME_STAMP_INDEX.fetch_add(1, Ordering::SeqCst)
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

    pub(crate) fn assign_heat_nr(&mut self, time_stamp_index: u64, heat_nr: u16) {
        if let Some(time_stamp) = self
            .time_stamps
            .iter_mut()
            .find(|time_stamp| time_stamp.index == time_stamp_index)
        {
            time_stamp.heat_nr = Some(heat_nr);
        }
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

    /// The heat number.
    pub(crate) heat_nr: Option<u16>,

    /// The heat number.
    pub(crate) bib: Option<u32>,
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
            heat_nr: None,
            bib: None,
        }
    }
}

/// The type of a time stamp.
#[derive(Debug, Clone, Display)]
pub(crate) enum TimeStampType {
    /// A start time stamp.
    #[strum(to_string = "Start")]
    Start,

    /// A finish time stamp.
    #[strum(to_string = "Ziel")]
    Finish,
}
