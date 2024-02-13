use std::time::Instant;

use actix_web::cookie::time::Date;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    generated: Instant,
    entries: Vec<ScheduleEntry>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleEntry {
    /// The race number
    race_number: String,
    /// The race short label (e.g. "MM 2x A-K")
    race_short_label: String,
    num_boats: u8,
    /// distance in meters
    distance: i32,
}
