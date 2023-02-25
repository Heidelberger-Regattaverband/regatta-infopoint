use super::{Club, RowColumn, ToEntity};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
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
    pub fn from_rows(rows: &Vec<Row>) -> Vec<Self> {
        let mut scores: Vec<Score> = Vec::with_capacity(rows.len());
        for row in rows {
            scores.push(row.to_entity());
        }
        scores
    }

    pub fn query_all<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new(
            "SELECT s.rank, s.points, c.Club_Name, c.Club_Abbr, c.Club_City
            FROM HRV_Score s
            JOIN Club AS c ON s.club_id = c.Club_ID
            WHERE s.event_id = @P1
            ORDER BY s.rank ASC",
        );
        query.bind(regatta_id);
        query
    }
}
