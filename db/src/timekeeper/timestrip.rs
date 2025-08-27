use crate::timekeeper::time_stamp::{Split, TimeStamp};
use log::info;
use serde::Serialize;

/// A time strip is a collection of time stamps.
#[derive(Default, Serialize)]
pub struct TimeStrip {
    // A vector of time stamps.
    pub time_stamps: Vec<TimeStamp>,
}

impl TimeStrip {
    pub fn load(regatta_id: i32) -> Self {
        info!("Loading time strip for regatta ID: {regatta_id}");
        // TODO load from DB
        TimeStrip { time_stamps: vec![] }
    }

    pub fn add_new_start(&mut self) {
        let time_stamp = TimeStamp::now(Split::Start);
        info!("Start time stamp: {time_stamp:?}");
        self.time_stamps.push(time_stamp);
    }

    pub fn add_new_finish(&mut self) {
        let time_stamp = TimeStamp::now(Split::Finish);
        info!("Finish time stamp: {time_stamp:?}");
        self.time_stamps.push(time_stamp);
    }

    pub fn assign_heat_nr(&mut self, time_stamp_index: u64, heat_nr: i16) -> Option<TimeStamp> {
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
