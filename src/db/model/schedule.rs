use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Schedule {
    /// The date and time when the schedule was generated
    generated: DateTime<Utc>,

    /// The schedule entries
    entries: Vec<ScheduleEntry>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScheduleEntry {
    /// The race number
    race_number: String,

    /// The race short label (e.g. "MM 2x A-K")
    race_short_label: String,

    /// The number of boats in the race
    num_boats: u8,

    /// distance in meters
    distance: i32,
}
