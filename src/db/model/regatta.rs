use super::{utils, RowColumn, ToEntity};
use crate::db::aquarius::AquariusClient;
use serde::Serialize;
use tiberius::{time::chrono::NaiveDateTime, Query, Row};

#[derive(Debug, Serialize, Clone)]
pub struct Regatta {
    pub id: i32,
    title: String,
    sub_title: String,
    venue: String,
    #[serde(rename = "startDate")]
    start_date: String,
    #[serde(rename = "endDate")]
    end_date: String,
}

impl ToEntity<Regatta> for Row {
    fn to_entity(&self) -> Regatta {
        let start_date: NaiveDateTime = self.get_column("Event_StartDate");
        let end_date: NaiveDateTime = self.get_column("Event_EndDate");

        Regatta {
            id: self.get_column("Event_ID"),
            title: self.get_column("Event_Title"),
            sub_title: self.get_column("Event_SubTitle"),
            venue: self.get_column("Event_Venue"),
            start_date: start_date.date().to_string(),
            end_date: end_date.date().to_string(),
        }
    }
}

impl Regatta {
    pub async fn query_all(client: &mut AquariusClient<'_>) -> Vec<Regatta> {
        let stream = Query::new("SELECT * FROM Event e")
            .query(client)
            .await
            .unwrap();
        let regattas = utils::get_rows(stream).await;
        regattas.into_iter().map(|row| row.to_entity()).collect()
    }

    pub async fn query(regatta_id: i32, client: &mut AquariusClient<'_>) -> Regatta {
        let mut query = Query::new("SELECT * FROM Event e WHERE e.Event_ID = @P1");
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        utils::get_row(stream).await.to_entity()
    }
}
