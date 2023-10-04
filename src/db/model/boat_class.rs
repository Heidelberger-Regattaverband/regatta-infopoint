use crate::db::{
    model::{ToEntity, TryToEntity},
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::Row;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BoatClass {
    id: i32,

    /// The caption of the boat class
    caption: String,

    /// An abbrevation of the boat class
    abbreviation: String,

    /// Number of rowers in the boat
    num_rowers: u8,

    /// Whether boat is coxed or not
    coxed: bool,
}

impl BoatClass {
    pub fn select_columns(alias: &str) -> String {
        format!(" {0}.BoatClass_ID, {0}.BoatClass_Caption, {0}.BoatClass_Abbr, {0}.BoatClass_NumRowers, {0}.BoatClass_Coxed ", alias)
    }
}

impl ToEntity<BoatClass> for Row {
    fn to_entity(&self) -> BoatClass {
        BoatClass {
            id: self.get_column("BoatClass_ID"),
            caption: self.get_column("BoatClass_Caption"),
            abbreviation: self.get_column("BoatClass_Abbr"),
            num_rowers: self.get_column("BoatClass_NumRowers"),
            coxed: <Row as RowColumn<u8>>::get_column(self, "BoatClass_Coxed") > 0,
        }
    }
}

impl TryToEntity<BoatClass> for Row {
    fn try_to_entity(&self) -> Option<BoatClass> {
        <Row as TryRowColumn<i32>>::try_get_column(self, "BoatClass_ID").map(|_id| self.to_entity())
    }
}
