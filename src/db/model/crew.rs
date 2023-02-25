use super::{Athlete, Column, RowColumn, RowToEntity};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
pub struct Crew {
    id: i32,
    pos: u8,
    cox: bool,
    athlete: Athlete,
}

impl RowToEntity<Crew> for Row {
    fn to_entity(&self) -> Crew {
        Crew {
            id: Column::get(self, "Crew_ID"),
            pos: self.get_column("Crew_Pos"),
            cox: self.get_column("Crew_IsCox"),
            athlete: self.to_entity(),
        }
    }
}

impl Crew {
    pub fn from_rows(rows: &Vec<Row>) -> Vec<Crew> {
        let mut crews: Vec<Crew> = Vec::with_capacity(rows.len());
        for row in rows {
            crews.push(row.to_entity());
        }
        crews
    }

    pub fn query_all<'a>(registration_id: i32) -> Query<'a> {
        let mut query = Query::new(
            "SELECT DISTINCT c.Crew_ID, c.Crew_Pos, c.Crew_IsCox,
            a.Athlet_ID, a.Athlet_FirstName, a.Athlet_LastName, a.Athlet_Gender, a.Athlet_DOB,
            cl.Club_UltraAbbr
            FROM Crew c
            JOIN Athlet AS a ON c.Crew_Athlete_ID_FK = a.Athlet_ID
            JOIN Club AS cl ON a.Athlet_Club_ID_FK = cl.Club_ID
            WHERE c.Crew_Entry_ID_FK = @P1 AND c.Crew_RoundFrom <= 64 AND 64 <= c.Crew_RoundTo
            ORDER BY c.Crew_pos ASC",
        );
        query.bind(registration_id);
        query
    }
}
