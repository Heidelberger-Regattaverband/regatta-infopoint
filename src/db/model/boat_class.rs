use super::TryToEntity;
use crate::db::tiberius::{RowColumn, TryRowColumn};
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

impl TryToEntity<BoatClass> for Row {
    fn try_to_entity(&self) -> Option<BoatClass> {
        if let Some(id) = self.try_get_column("BoatClass_ID") {
            let coxed: u8 = self.get_column("BoatClass_Coxed");
            Some(BoatClass {
                id,
                caption: self.get_column("BoatClass_Caption"),
                abbreviation: self.get_column("BoatClass_Abbr"),
                num_rowers: self.get_column("BoatClass_NumRowers"),
                coxed: coxed > 0,
            })
        } else {
            None
        }
    }
}
