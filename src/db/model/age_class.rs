use super::{Column, RowColumn, TryToEntity};
use serde::Serialize;
use tiberius::Row;

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

impl TryToEntity<AgeClass> for Row {
    fn try_to_entity(&self) -> Option<AgeClass> {
        let it: Option<i32> = Column::get(self, "AgeClass_ID");
        it.map(|id| AgeClass {
            id,
            caption: self.get_column("AgeClass_Caption"),
            abbreviation: self.get_column("AgeClass_Abbr"),
            suffix: self.get_column("AgeClass_Suffix"),
            gender: self.get_column("AgeClass_Gender"),
            num_sub_classes: self.get_column("AgeClass_NumSubClasses"),
        })
    }
}
