use crate::{
    aquarius::model::TryToEntity,
    tiberius::{RowColumn, TryRowColumn},
};
use ::serde::Serialize;
use ::tiberius::Row;
use ::utoipa::ToSchema;

pub(super) const ID: &str = "BoatClass_ID";
const CAPTION: &str = "BoatClass_Caption";
const ABBREVIATION: &str = "BoatClass_Abbr";
pub(super) const NUM_ROWERS: &str = "BoatClass_NumRowers";
pub(super) const COXED: &str = "BoatClass_Coxed";

/// A boat class is a combination of boat type and number of rowers.
#[derive(Debug, Serialize, Clone, ToSchema)]
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
    pub(crate) fn select_columns(alias: &str) -> String {
        format!("{alias}.{ID}, {alias}.{CAPTION}, {alias}.{ABBREVIATION}, {alias}.{NUM_ROWERS}, {alias}.{COXED}")
    }
}

impl From<&Row> for BoatClass {
    fn from(row: &Row) -> Self {
        BoatClass {
            id: row.get_column(ID),
            caption: row.get_column(CAPTION),
            abbreviation: row.get_column(ABBREVIATION),
            num_rowers: row.get_column(NUM_ROWERS),
            coxed: <Row as RowColumn<u8>>::get_column(row, COXED) > 0,
        }
    }
}

impl TryToEntity<BoatClass> for Row {
    fn try_to_entity(&self) -> Option<BoatClass> {
        <Row as TryRowColumn<i32>>::try_get_column(self, ID).map(|_id| BoatClass::from(self))
    }
}
