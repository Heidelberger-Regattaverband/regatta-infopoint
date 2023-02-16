use crate::db::utils::Column;
use serde::Serialize;
use tiberius::{time::chrono::NaiveDateTime, Query, Row};

#[derive(Debug, Serialize, Clone)]
pub struct Athlete {
    id: i32,
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
    gender: String,
    year: String,
    club: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Crew {
    id: i32,
    pos: u8,
    cox: bool,
    athlete: Athlete,
}

impl Crew {
    pub fn from_rows(rows: &Vec<Row>) -> Vec<Crew> {
        let mut crews: Vec<Crew> = Vec::with_capacity(rows.len());
        for crew_row in rows {
            crews.push(Crew::from_row(crew_row));
        }
        crews
    }

    fn from_row(row: &Row) -> Crew {
        let dob: NaiveDateTime = Column::get(row, "Athlet_DOB");

        Crew {
            id: Column::get(row, "Crew_ID"),
            pos: Column::get(row, "Crew_Pos"),
            cox: Column::get(row, "Crew_IsCox"),
            athlete: Athlete {
                id: Column::get(row, "Athlet_ID"),
                first_name: Column::get(row, "Athlet_FirstName"),
                last_name: Column::get(row, "Athlet_LastName"),
                gender: Column::get(row, "Athlet_Gender"),
                year: dob.date().format("%Y").to_string(),
                club: Column::get(row, "Club_UltraAbbr"),
            },
        }
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
