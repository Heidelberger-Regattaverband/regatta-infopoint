use crate::db::{
    aquarius::AquariusClient,
    model::{utils, ToEntity},
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::{time::chrono::NaiveDateTime, Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Regatta {
    pub id: i32,
    title: String,
    sub_title: String,
    venue: String,
    start_date: String,
    end_date: String,
}

impl ToEntity<Regatta> for Row {
    fn to_entity(&self) -> Regatta {
        let start_date: NaiveDateTime = self.get_column("Event_StartDate");
        let end_date: NaiveDateTime = self.get_column("Event_EndDate");

        Regatta {
            id: self.get_column("Event_ID"),
            title: self.get_column("Event_Title"),
            sub_title: self.try_get_column("Event_SubTitle").unwrap_or_default(),
            venue: self.try_get_column("Event_Venue").unwrap_or_default(),
            start_date: start_date.date().to_string(),
            end_date: end_date.date().to_string(),
        }
    }
}

impl Regatta {
    pub async fn query_all(client: &mut AquariusClient<'_>) -> Vec<Regatta> {
        let stream = Query::new("SELECT * FROM Event").query(client).await.unwrap();
        let regattas = utils::get_rows(stream).await;
        regattas.into_iter().map(|row| row.to_entity()).collect()
    }

    pub async fn query(regatta_id: i32, client: &mut AquariusClient<'_>) -> Regatta {
        let mut query = Query::new("SELECT * FROM Event WHERE Event_ID = @P1");
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        utils::get_row(stream).await.to_entity()
    }
}
