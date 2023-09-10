use crate::db::{
    model::{ToEntity, TryToEntity},
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::Row;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgeClass {
    id: i32,
    caption: String,
    abbreviation: String,
    suffix: String,
    gender: String,
    num_sub_classes: u8,
}

impl ToEntity<AgeClass> for Row {
    fn to_entity(&self) -> AgeClass {
        AgeClass {
            id: self.get_column("AgeClass_ID"),
            caption: self.get_column("AgeClass_Caption"),
            abbreviation: self.get_column("AgeClass_Abbr"),
            suffix: self.try_get_column("AgeClass_Suffix").unwrap_or_default(),
            gender: self.get_column("AgeClass_Gender"),
            num_sub_classes: self.get_column("AgeClass_NumSubClasses"),
        }
    }
}

impl TryToEntity<AgeClass> for Row {
    fn try_to_entity(&self) -> Option<AgeClass> {
        <Row as TryRowColumn<i32>>::try_get_column(self, "AgeClass_ID").map(|id| AgeClass {
            id,
            caption: self.get_column("AgeClass_Caption"),
            abbreviation: self.get_column("AgeClass_Abbr"),
            suffix: self.try_get_column("AgeClass_Suffix").unwrap_or_default(),
            gender: self.get_column("AgeClass_Gender"),
            num_sub_classes: self.get_column("AgeClass_NumSubClasses"),
        })
    }
}
