use tiberius::{QueryStream, Row, error::Error as DbError};

pub async fn get_row(stream: QueryStream<'_>) -> Row {
    stream.into_row().await.unwrap().unwrap()
}

pub async fn try_get_row(stream: QueryStream<'_>) -> Option<Row> {
    stream.into_row().await.unwrap()
}

pub async fn get_rows(stream: QueryStream<'_>) -> Result<Vec<Row>, DbError> {
    Ok(stream.into_first_result().await?)
}
