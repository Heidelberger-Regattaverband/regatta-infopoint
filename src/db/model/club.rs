use crate::db::{
    aquarius::AquariusClient,
    model::{utils, ToEntity},
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Club {
    id: i32,
    short_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    long_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    abbreviation: Option<String>,
    city: String,
}

impl Club {
    pub async fn query_participating(regatta_id: i32, client: &mut AquariusClient<'_>) -> Vec<Club> {
        let mut query = Query::new(
            "SELECT DISTINCT Club_ID, Club_Name, Club_Abbr, Club_UltraAbbr, Club_City
            FROM Club
            JOIN Athlet ON Athlet_Club_ID_FK  = Club_ID
            JOIN Crew   ON Crew_Athlete_ID_FK = Athlet_ID
            JOIN Entry  ON Crew_Entry_ID_FK   = Entry_ID
            JOIN Event  ON Entry_Event_ID_FK  = Event_ID
            WHERE Event_ID = @P1
            ORDER BY Club_City ASC",
        );
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let regattas = utils::get_rows(stream).await;
        regattas.into_iter().map(|row| row.to_entity()).collect()
    }
}

impl ToEntity<Club> for Row {
    fn to_entity(&self) -> Club {
        Club {
            id: self.get_column("Club_ID"),
            short_name: self.get_column("Club_Abbr"),
            long_name: self.try_get_column("Club_Name"),
            abbreviation: self.try_get_column("Club_UltraAbbr"),
            city: self.get_column("Club_City"),
        }
    }
}
