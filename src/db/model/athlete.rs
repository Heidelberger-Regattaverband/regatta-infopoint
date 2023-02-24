use super::column::{Column, RowToEntity};
use serde::Serialize;
use tiberius::{time::chrono::NaiveDateTime, Row};

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

impl RowToEntity<Athlete> for Row {
    fn to_entity(&self) -> Athlete {
        let dob: NaiveDateTime = Column::get(self, "Athlet_DOB");
        Athlete {
            id: Column::get(self, "Athlet_ID"),
            first_name: Column::get(self, "Athlet_FirstName"),
            last_name: Column::get(self, "Athlet_LastName"),
            gender: Column::get(self, "Athlet_Gender"),
            year: dob.date().format("%Y").to_string(),
            club: Column::get(self, "Club_UltraAbbr"),
        }
    }
}
