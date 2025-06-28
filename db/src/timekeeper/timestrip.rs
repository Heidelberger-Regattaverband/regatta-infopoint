use crate::timekeeper::time_stamp::{TimeStamp, TimeStampType};
use log::info;

/// A time strip is a collection of time stamps.
#[derive(Default)]
pub struct TimeStrip {
    // A vector of time stamps.
    pub time_stamps: Vec<TimeStamp>,
}

impl TimeStrip {
    pub fn add_new_start(&mut self) {
        let time_stamp = TimeStamp::now(TimeStampType::Start);
        info!("Start time stamp: {time_stamp:?}");
        self.time_stamps.push(time_stamp);
    }

    pub fn add_new_finish(&mut self) {
        let time_stamp = TimeStamp::now(TimeStampType::Finish);
        info!("Finish time stamp: {time_stamp:?}");
        self.time_stamps.push(time_stamp);
    }

    pub fn assign_heat_nr(&mut self, time_stamp_index: u64, heat_nr: u16) -> Option<TimeStamp> {
        if let Some(time_stamp) = self
            .time_stamps
            .iter_mut()
            .find(|time_stamp| time_stamp.index == time_stamp_index)
        {
            time_stamp.heat_nr = Some(heat_nr);
            return Some(time_stamp.clone());
        }
        None
    }
}
