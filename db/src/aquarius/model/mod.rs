mod age_class;
mod athlete;
mod block;
mod boat_class;
mod club;
mod crew;
mod entry;
mod filters;
mod heat;
mod heat_entry;
mod heat_result;
mod notification;
mod race;
mod referee;
mod regatta;
mod schedule;
mod score;
mod statistics;

use crate::error::DbError;
use ::tiberius::QueryStream;
use ::tiberius::Row;
use ::tiberius::error::Error as TiberiusError;
pub use age_class::AgeClass;
pub use athlete::Athlete;
pub use block::Block;
pub use boat_class::BoatClass;
pub use club::Club;
pub use crew::Crew;
pub use entry::Entry;
pub use filters::Filters;
pub use heat::Heat;
pub use heat_entry::HeatEntry;
pub use heat_result::HeatResult;
pub use notification::{CreateNotificationRequest, Notification, UpdateNotificationRequest};
pub use race::Race;
pub use referee::Referee;
pub use regatta::Regatta;
pub use schedule::{Schedule, ScheduleEntry};
pub use score::Score;
pub use statistics::Statistics;

pub trait TryToEntity<T> {
    fn try_to_entity(&self) -> Option<T>;
}

/// Extract a single row from a query stream, returning an error if no row is found.
///
/// # Errors
/// Returns an error if the query fails or if no row is returned.
pub async fn get_row(stream: QueryStream<'_>) -> Result<Row, DbError> {
    stream
        .into_row()
        .await?
        .ok_or_else(|| DbError::from(TiberiusError::Conversion("No row returned from query".into())))
}

/// Extract an optional row from a query stream.
///
/// # Errors
/// Returns an error if the query fails, but returns `Ok(None)` if no row is found.
pub async fn try_get_row(stream: QueryStream<'_>) -> Result<Option<Row>, DbError> {
    stream.into_row().await.map_err(DbError::from)
}

/// Extract all rows from a query stream.
///
/// # Errors
/// Returns an error if the query fails.
pub async fn get_rows(stream: QueryStream<'_>) -> Result<Vec<Row>, DbError> {
    stream.into_first_result().await.map_err(DbError::from)
}
