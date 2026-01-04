use crate::{
    aquarius::model::TryToEntity,
    tiberius::{RowColumn, TryRowColumn},
};
use ::serde::Serialize;
use ::tiberius::Row;
use ::utoipa::ToSchema;

pub(super) const ID: &str = "AgeClass_ID";
const CAPTION: &str = "AgeClass_Caption";
const ABBREVIATION: &str = "AgeClass_Abbr";
const SUFFIX: &str = "AgeClass_Suffix";
const GENDER: &str = "AgeClass_Gender";
const NUM_SUB_CLASSES: &str = "AgeClass_NumSubClasses";
pub(super) const MIN_AGE: &str = "AgeClass_MinAge";
pub(super) const MAX_AGE: &str = "AgeClass_MaxAge";

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
    pub(crate) fn select_all_columns(alias: &str) -> String {
        format!(
            "{alias}.{ID}, {alias}.{CAPTION}, {alias}.{ABBREVIATION}, {alias}.{SUFFIX}, {alias}.{GENDER}, \
            {alias}.{NUM_SUB_CLASSES}, {alias}.{MIN_AGE}, {alias}.{MAX_AGE}"
        )
    }
    pub(crate) fn select_minimal_columns(alias: &str) -> String {
        format!(
            "{alias}.{ID}, {alias}.{CAPTION}, {alias}.{ABBREVIATION}, {alias}.{SUFFIX}, {alias}.{GENDER}, \
            {alias}.{NUM_SUB_CLASSES}"
        )
    }
}

impl From<&Row> for AgeClass {
    fn from(row: &Row) -> Self {
        AgeClass {
            id: row.get_column(ID),
            caption: row.get_column(CAPTION),
            abbreviation: row.get_column(ABBREVIATION),
            suffix: row.try_get_column(SUFFIX).unwrap_or_default(),
            gender: row.get_column(GENDER),
            num_sub_classes: row.get_column(NUM_SUB_CLASSES),
            min_age: row.try_get_column(MIN_AGE),
            max_age: row.try_get_column(MAX_AGE),
        }
    }
}

impl TryToEntity<AgeClass> for Row {
    fn try_to_entity(&self) -> Option<AgeClass> {
        <Row as TryRowColumn<i32>>::try_get_column(self, ID).map(|_id| AgeClass::from(self))
    }
}
