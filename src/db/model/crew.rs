use crate::db::{
    aquarius::AquariusClient,
    model::{utils, Athlete, ToEntity},
    tiberius::RowColumn,
};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Crew {
    id: i32,
    pos: u8,
    cox: bool,
    athlete: Athlete,
}

impl ToEntity<Crew> for Row {
    fn to_entity(&self) -> Crew {
        Crew {
            id: self.get_column("Crew_ID"),
            pos: self.get_column("Crew_Pos"),
            cox: self.get_column("Crew_IsCox"),
            athlete: self.to_entity(),
        }
    }
}

impl Crew {
    pub async fn query_all<'a>(registration_id: i32, client: &mut AquariusClient<'_>) -> Vec<Crew> {
        let mut query = Query::new(
            "SELECT DISTINCT Crew.*, Athlet.*, Club.Club_ID, Club.Club_Abbr, Club.Club_UltraAbbr, Club.Club_City
            FROM Crew
            JOIN Athlet ON Crew_Athlete_ID_FK = Athlet_ID
            JOIN Club   ON Athlet_Club_ID_FK  = Club_ID
            WHERE Crew_Entry_ID_FK = @P1 AND Crew_RoundFrom <= 64 AND 64 <= Crew_RoundTo
            ORDER BY Crew_pos ASC",
        );
        query.bind(registration_id);
        let stream = query.query(client).await.unwrap();
        let crew = utils::get_rows(stream).await;
        crew.into_iter().map(|row| row.to_entity()).collect()
    }
}
