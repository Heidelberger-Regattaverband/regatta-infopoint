use crate::db::{
    model::TryToEntity,
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::Row;

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Referee {
    id: i32,
    first_name: String,
    last_name: String,
}

impl TryToEntity<Referee> for Row {
    fn try_to_entity(&self) -> Option<Referee> {
        if let Some(id) = self.try_get_column("Referee_ID") {
            let last_name: String = self.get_column("Referee_LastName");
            let first_name: String = self.get_column("Referee_FirstName");
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
