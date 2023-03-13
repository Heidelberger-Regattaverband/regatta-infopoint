use super::ToEntity;
use crate::db::tiberius::RowColumn;
use serde::Serialize;
use tiberius::Row;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Club {
    id: i32,
    short_name: String,
    city: String,
}

impl ToEntity<Club> for Row {
    fn to_entity(&self) -> Club {
        Club {
            id: self.get_column("Club_ID"),
            short_name: self.get_column("Club_Abbr"),
            city: self.get_column("Club_City"),
        }
    }
}
