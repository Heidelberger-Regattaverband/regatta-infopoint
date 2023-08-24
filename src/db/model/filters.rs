use crate::db::{aquarius::AquariusClient, model::utils, tiberius::RowColumn};
use chrono::NaiveDate;
use serde::Serialize;
use tiberius::Query;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Filters {
    /// Available distances of all races.
    distance: Vec<i16>,

    /// All dates of the races.
    date: Vec<NaiveDate>,
}

impl Filters {
    pub async fn query(regatta_id: i32, client: &mut AquariusClient<'_>) -> Self {
        // get all available distances
        let mut query = Query::new("SELECT DISTINCT Offer_Distance FROM Offer WHERE Offer_Event_ID_FK = @P1");
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let rows = utils::get_rows(stream).await;
        let distance = rows.into_iter().map(|row| row.get_column("Offer_Distance")).collect();

        // get all available dates
        let mut query = Query::new(
            "SELECT DISTINCT CAST (c.Comp_Datetime as date) AS Comp_Date FROM Comp c WHERE c.Comp_Event_ID_FK = @P1",
        );
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let rows = utils::get_rows(stream).await;
        let date = rows.into_iter().map(|row| row.get_column("Comp_Date")).collect();

        Filters { distance, date }
    }
}
