use tiberius::{QueryStream, Row};

pub async fn get_row(stream: QueryStream<'_>) -> Row {
    stream.into_row().await.unwrap().unwrap()
}

pub async fn try_get_row(stream: QueryStream<'_>) -> Option<Row> {
    stream.into_row().await.unwrap()
}

pub async fn get_rows(stream: QueryStream<'_>) -> Vec<Row> {
    stream.into_first_result().await.unwrap()
}
