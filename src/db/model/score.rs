use crate::db::{
    aquarius::AquariusClient,
    model::{utils, Club, ToEntity},
    tiberius::RowColumn,
};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    rank: i16,
    points: f64,
    club: Club,
}

impl ToEntity<Score> for Row {
    fn to_entity(&self) -> Score {
        Score {
            rank: self.get_column("rank"),
            points: self.get_column("points"),
            club: self.to_entity(),
        }
    }
}

impl Score {
    pub async fn query_all<'a>(regatta_id: i32, client: &mut AquariusClient<'_>) -> Vec<Self> {
        let mut query = Query::new(
            "SELECT s.rank, s.points, c.Club_ID, c.Club_Name, c.Club_Abbr, c.Club_City
            FROM HRV_Score AS s
            JOIN Club      AS c ON s.club_id = c.Club_ID
            WHERE s.event_id = @P1
            ORDER BY s.rank ASC",
        );
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let scores = utils::get_rows(stream).await;
        scores.into_iter().map(|row| row.to_entity()).collect()
    }
}
