use tiberius::{QueryStream, Row};

pub async fn get_row(stream: QueryStream<'_>) -> Row {
    stream.into_row().await.unwrap().unwrap()
}

pub async fn get_rows(stream: QueryStream<'_>) -> Vec<Row> {
    stream.into_first_result().await.unwrap()
}
