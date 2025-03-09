use tiberius::{QueryStream, Row, error::Error as DbError};

pub async fn get_row(stream: QueryStream<'_>) -> Result<Row, DbError> {
    Ok(stream.into_row().await?.unwrap())
}

pub async fn try_get_row(stream: QueryStream<'_>) -> Result<Option<Row>, DbError> {
    stream.into_row().await
}

pub async fn get_rows(stream: QueryStream<'_>) -> Result<Vec<Row>, DbError> {
    stream.into_first_result().await
}
