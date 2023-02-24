use super::{column::RowToEntity, Column};
use serde::Serialize;
use tiberius::Row;

#[derive(Debug, Serialize, Clone)]
pub struct Club {
    id: i32,
    #[serde(rename = "shortName")]
    short_name: String,
    city: String,
}

impl RowToEntity<Club> for Row {
    fn to_entity(&self) -> Club {
        Club {
            id: Column::get(self, "Club_ID"),
            short_name: Column::get(self, "Club_Abbr"),
            city: Column::get(self, "Club_City"),
        }
    }
}
