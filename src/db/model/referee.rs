use super::{Column, TryToEntity};
use serde::Serialize;
use tiberius::Row;

#[derive(Debug, Serialize, Clone, Default)]
pub struct Referee {
    id: i32,
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
}

impl TryToEntity<Referee> for Row {
    fn try_to_entity(&self) -> Option<Referee> {
        if let Some(id) = Column::get(self, "Referee_ID") {
            let last_name: String = Column::get(self, "Referee_LastName");
            let first_name: String = Column::get(self, "Referee_FirstName");
            if last_name.is_empty() && first_name.is_empty() {
                return None;
            }
            Some(Referee {
                id,
                last_name,
                first_name,
            })
        } else {
            None
        }
    }
}
