use crate::db::{aquarius::AquariusClient, model::utils, tiberius::RowColumn};
use serde::Serialize;
use tiberius::Query;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Filters {
    distances: Vec<i16>,
}

impl Filters {
    pub async fn query(regatta_id: i32, client: &mut AquariusClient<'_>) -> Self {
        let mut query = Query::new("SELECT DISTINCT Offer_Distance FROM Offer WHERE Offer_Event_ID_FK = @P1");
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let rows = utils::get_rows(stream).await;
        let distances = rows.into_iter().map(|row| row.get_column("Offer_Distance")).collect();
        Filters { distances }
    }
}
