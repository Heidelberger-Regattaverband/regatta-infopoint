use chrono::{DateTime, Utc};

/// A time strip is a collection of time stamps.
pub(crate) struct TimeStrip {
    // A vector of time stamps.
    pub(crate) time_stamps: Vec<TimeStamp>,
}

/// A time stamp of an event.
pub(crate) struct TimeStamp {
    /// The unique identifier of the time stamp.
    pub(crate) id: u32,

    /// The time of the event.
    pub(crate) time: DateTime<Utc>,

    /// The type of the time stamp.
    pub(crate) stamp_type: TimeStampType,
}

/// The type of a time stamp.
pub(crate) enum TimeStampType {
    Start,
    Finish,
}
