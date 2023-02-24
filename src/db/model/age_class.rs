use serde::Serialize;
use tiberius::Row;

use super::{column::TryRowToEntity, Column};

#[derive(Debug, Serialize, Clone)]
pub struct AgeClass {
    id: i32,
    caption: String,
    abbreviation: String,
    suffix: String,
    gender: String,
    #[serde(rename = "numSubClasses")]
    num_sub_classes: u8,
}

impl TryRowToEntity<AgeClass> for Row {
    fn try_to_entity(&self) -> Option<AgeClass> {
        if let Some(id) = Column::get(self, "AgeClass_ID") {
            Some(AgeClass {
                id,
                caption: Column::get(self, "AgeClass_Caption"),
                abbreviation: Column::get(self, "AgeClass_Abbr"),
                suffix: Column::get(self, "AgeClass_Suffix"),
                gender: Column::get(self, "AgeClass_Gender"),
                num_sub_classes: Column::get(self, "AgeClass_NumSubClasses"),
            })
        } else {
            None
        }
    }
}
