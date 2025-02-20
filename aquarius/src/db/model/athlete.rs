use crate::db::{
    model::{Club, TryToEntity},
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::{Row, time::chrono::NaiveDateTime};

/// An athlete is a person who participates in a regatta.
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Athlete {
    /// The internal ID of the athlete.
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

impl From<&Row> for Athlete {
    fn from(row: &Row) -> Self {
        Athlete {
            id: row.get_column("Athlet_ID"),
            first_name: row.get_column("Athlet_FirstName"),
            last_name: row.get_column("Athlet_LastName"),
            gender: row.get_column("Athlet_Gender"),
            year: <Row as RowColumn<NaiveDateTime>>::get_column(row, "Athlet_DOB")
                .date()
                .format("%Y")
                .to_string(),
            club: Club::from(row),
        }
    }
}

impl TryToEntity<Athlete> for Row {
    fn try_to_entity(&self) -> Option<Athlete> {
        <Row as TryRowColumn<i32>>::try_get_column(self, "Athlet_ID").map(|_id| Athlete::from(self))
    }
}
