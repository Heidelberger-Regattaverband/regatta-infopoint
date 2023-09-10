use crate::db::{
    model::TryToEntity,
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::Row;

use super::ToEntity;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BoatClass {
    id: i32,

    /// The caption of the boat class
    caption: String,

    /// An abbrevation of the boat class
    abbreviation: String,

    /// Number of rowers in the boat
    num_rowers: i32,

    /// Whether boat is coxed or not
    coxed: bool,
}

impl ToEntity<BoatClass> for Row {
    fn to_entity(&self) -> BoatClass {
        let coxed: u8 = self.get_column("BoatClass_Coxed");
        BoatClass {
            id: self.get_column("BoatClass_ID"),
            caption: self.get_column("BoatClass_Caption"),
            abbreviation: self.get_column("BoatClass_Abbr"),
            num_rowers: self.try_get_column("BoatClass_NumRowers").unwrap_or_default(),
            coxed: coxed > 0,
        }
    }
}

impl TryToEntity<BoatClass> for Row {
    fn try_to_entity(&self) -> Option<BoatClass> {
        if let Some(id) = self.try_get_column("BoatClass_ID") {
            let coxed: u8 = self.get_column("BoatClass_Coxed");
            Some(BoatClass {
                id,
                caption: self.get_column("BoatClass_Caption"),
                abbreviation: self.get_column("BoatClass_Abbr"),
                num_rowers: self.try_get_column("BoatClass_NumRowers").unwrap_or_default(),
                coxed: coxed > 0,
            })
        } else {
            None
        }
    }
}
