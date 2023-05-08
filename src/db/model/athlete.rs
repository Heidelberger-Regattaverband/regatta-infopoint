use crate::db::{model::ToEntity, tiberius::RowColumn};
use serde::Serialize;
use tiberius::{time::chrono::NaiveDateTime, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Athlete {
    id: i32,
    first_name: String,
    last_name: String,
    gender: String,
    year: String,
    club: String,
}

impl ToEntity<Athlete> for Row {
    fn to_entity(&self) -> Athlete {
        let dob: NaiveDateTime = self.get_column("Athlet_DOB");
        Athlete {
            id: self.get_column("Athlet_ID"),
            first_name: self.get_column("Athlet_FirstName"),
            last_name: self.get_column("Athlet_LastName"),
            gender: self.get_column("Athlet_Gender"),
            year: dob.date().format("%Y").to_string(),
            club: self.get_column("Club_UltraAbbr"),
        }
    }
}
