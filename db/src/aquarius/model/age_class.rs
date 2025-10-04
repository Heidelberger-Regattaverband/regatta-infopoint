use crate::{
    aquarius::model::TryToEntity,
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::Row;
use utoipa::ToSchema;

/// An age class defines the age range of athletes.
#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgeClass {
    /// The internal ID of the age class.
    id: i32,

    /// A long and human readable caption of this age class.
    caption: String,

    /// An abbreviation of this age class.
    abbreviation: String,

    /// An suffix to this age class.
    suffix: String,

    /// The gender of this age class. Known values are: "M" (male), "W" (female) or "X" (mixed).
    gender: String,

    /// The number of sub-classes, e.g. for masters age classes
    num_sub_classes: u8,

    /// The minimum age in this class
    #[serde(skip_serializing_if = "Option::is_none")]
    min_age: Option<u8>,

    /// The maximum age in this class
    #[serde(skip_serializing_if = "Option::is_none")]
    max_age: Option<u8>,
}

impl AgeClass {
    pub fn select_all_columns(alias: &str) -> String {
        format!(
            " {alias}.AgeClass_ID, {alias}.AgeClass_Caption, {alias}.AgeClass_Abbr, {alias}.AgeClass_Suffix, {alias}.AgeClass_Gender, \
            {alias}.AgeClass_NumSubClasses, {alias}.AgeClass_MinAge, {alias}.AgeClass_MaxAge "
        )
    }
    pub fn select_minimal_columns(alias: &str) -> String {
        format!(
            " {alias}.AgeClass_ID, {alias}.AgeClass_Caption, {alias}.AgeClass_Abbr, {alias}.AgeClass_Suffix, {alias}.AgeClass_Gender, \
            {alias}.AgeClass_NumSubClasses "
        )
    }
}

impl From<&Row> for AgeClass {
    fn from(row: &Row) -> Self {
        AgeClass {
            id: row.get_column("AgeClass_ID"),
            caption: row.get_column("AgeClass_Caption"),
            abbreviation: row.get_column("AgeClass_Abbr"),
            suffix: row.try_get_column("AgeClass_Suffix").unwrap_or_default(),
            gender: row.get_column("AgeClass_Gender"),
            num_sub_classes: row.get_column("AgeClass_NumSubClasses"),
            min_age: row.try_get_column("AgeClass_MinAge"),
            max_age: row.try_get_column("AgeClass_MaxAge"),
        }
    }
}

impl TryToEntity<AgeClass> for Row {
    fn try_to_entity(&self) -> Option<AgeClass> {
        <Row as TryRowColumn<i32>>::try_get_column(self, "AgeClass_ID").map(|_id| AgeClass::from(self))
    }
}
