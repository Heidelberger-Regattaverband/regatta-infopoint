use crate::db::{
    model::TryToEntity,
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::Row;

#[derive(Debug, Serialize, Clone)]
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
    min_age: u8,

    /// The maximum age in this class
    max_age: u8,

    /// Inidicates whether this age class is masters or not.
    is_masters: bool,
}

impl AgeClass {
    pub fn select_columns(alias: &str) -> String {
        format!(" {0}.AgeClass_ID, {0}.AgeClass_Caption, {0}.AgeClass_Abbr, {0}.AgeClass_Suffix, {0}.AgeClass_Gender, {0}.AgeClass_NumSubClasses, {0}.AgeClass_MinAge, {0}.AgeClass_MaxAge ", alias)
    }
}

impl From<&Row> for AgeClass {
    fn from(row: &Row) -> Self {
        let abbreviation: String = row.get_column("AgeClass_Abbr");
        let is_masters = matches!(abbreviation.as_str(), "MM" | "MW" | "MM/W");

        AgeClass {
            id: row.get_column("AgeClass_ID"),
            caption: row.get_column("AgeClass_Caption"),
            abbreviation,
            suffix: row.try_get_column("AgeClass_Suffix").unwrap_or_default(),
            gender: row.get_column("AgeClass_Gender"),
            num_sub_classes: row.get_column("AgeClass_NumSubClasses"),
            min_age: row.get_column("AgeClass_MinAge"),
            max_age: row.get_column("AgeClass_MaxAge"),
            is_masters,
        }
    }
}

impl TryToEntity<AgeClass> for Row {
    fn try_to_entity(&self) -> Option<AgeClass> {
        <Row as TryRowColumn<i32>>::try_get_column(self, "AgeClass_ID").map(|_id| AgeClass::from(self))
    }
}
