use super::{Column, RowColumn, TryRowToEntity};
use serde::Serialize;
use tiberius::Row;

#[derive(Debug, Serialize, Clone)]
pub struct BoatClass {
    id: i32,
    caption: String,
    abbreviation: String,
    #[serde(rename = "numRowers")]
    num_rowers: i32,
    coxed: bool,
}

impl TryRowToEntity<BoatClass> for Row {
    fn try_to_entity(&self) -> Option<BoatClass> {
        if let Some(id) = Column::get(self, "BoatClass_ID") {
            let coxed: u8 = self.get_column("BoatClass_Coxed");
            Some(BoatClass {
                id,
                caption: Column::get(self, "BoatClass_Caption"),
                abbreviation: Column::get(self, "BoatClass_Abbr"),
                num_rowers: Column::get(self, "BoatClass_NumRowers"),
                coxed: coxed > 0,
            })
        } else {
            None
        }
    }
}
