use crate::db::{
    model::{Club, ToEntity, TryToEntity},
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::{time::chrono::NaiveDateTime, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Athlete {
    id: i32,

    /// First name of the athlete.
    first_name: String,

    /// Last name of the athlete.
    last_name: String,

    /// The athlete's gender.
    gender: String,

    /// Year of birth.
    year: String,

    /// The athlete's club.
    club: Club,
}

impl Athlete {
    pub fn select_columns(alias: &str) -> String {
        format!(
            " {0}.Athlet_ID, {0}.Athlet_FirstName, {0}.Athlet_LastName, {0}.Athlet_Gender, {0}.Athlet_DOB ",
            alias
        )
    }
}

impl ToEntity<Athlete> for Row {
    fn to_entity(&self) -> Athlete {
        Athlete {
            id: self.get_column("Athlet_ID"),
            first_name: self.get_column("Athlet_FirstName"),
            last_name: self.get_column("Athlet_LastName"),
            gender: self.get_column("Athlet_Gender"),
            year: <Row as RowColumn<NaiveDateTime>>::get_column(self, "Athlet_DOB")
                .date()
                .format("%Y")
                .to_string(),
            club: self.to_entity(),
        }
    }
}

impl TryToEntity<Athlete> for Row {
    fn try_to_entity(&self) -> Option<Athlete> {
        <Row as TryRowColumn<i32>>::try_get_column(self, "Athlet_ID").map(|_id| self.to_entity())
    }
}
