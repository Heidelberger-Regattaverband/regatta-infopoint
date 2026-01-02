use crate::error::DbError;
use tiberius::{QueryStream, Row, error::Error as TiberiusError};

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
