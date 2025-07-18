use crate::{
    aquarius::model::TryToEntity,
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::Row;

/// A boat class is a combination of boat type and number of rowers.
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BoatClass {
    /// The internal ID of the boat class.
    id: i32,

    /// The caption of the boat class
    caption: String,

    /// An abbreviation of the boat class
    abbreviation: String,

    /// Number of rowers in the boat
    num_rowers: u8,

    /// Whether boat is coxed or not
    coxed: bool,
}

impl BoatClass {
    pub fn select_columns(alias: &str) -> String {
        format!(
            " {alias}.BoatClass_ID, {alias}.BoatClass_Caption, {alias}.BoatClass_Abbr, {alias}.BoatClass_NumRowers, {alias}.BoatClass_Coxed "
        )
    }
}

impl From<&Row> for BoatClass {
    fn from(row: &Row) -> Self {
        BoatClass {
            id: row.get_column("BoatClass_ID"),
            caption: row.get_column("BoatClass_Caption"),
            abbreviation: row.get_column("BoatClass_Abbr"),
            num_rowers: row.get_column("BoatClass_NumRowers"),
            coxed: <Row as RowColumn<u8>>::get_column(row, "BoatClass_Coxed") > 0,
        }
    }
}

impl TryToEntity<BoatClass> for Row {
    fn try_to_entity(&self) -> Option<BoatClass> {
        <Row as TryRowColumn<i32>>::try_get_column(self, "BoatClass_ID").map(|_id| BoatClass::from(self))
    }
}
