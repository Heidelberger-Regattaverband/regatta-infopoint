use crate::db::{
    model::TryToEntity,
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use std::time::Duration;
use tiberius::Row;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeatResult {
    rank_sort: u8,
    rank_label: String,
    result: String,

    /// The net time of the boat
    pub net_time: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    delta: Option<String>,

    /// The points given for the result
    points: u8,
}

impl HeatResult {
    pub fn select_columns(alias: &str) -> String {
        format!(
            " {0}.Result_Rank, {0}.Result_Delta, {0}.Result_DisplayValue, {0}.Result_NetTime ",
            alias
        )
    }
}

impl TryToEntity<HeatResult> for Row {
    fn try_to_entity(&self) -> Option<HeatResult> {
        if let Some(rank) = self.try_get_column("Result_Rank") {
            let rank_sort: u8 = if rank == 0 { u8::MAX } else { rank };
            let delta: Option<String> = if rank > 1 {
                let delta: i32 = self.get_column("Result_Delta");
                let duration = Duration::from_millis(delta as u64);
                let seconds = duration.as_secs();
                let millis = duration.subsec_millis() / 10;
                Some(format!("+{seconds}.{millis}"))
            } else {
                None
            };

            let rank_label: String = if rank == 0 {
                Default::default()
            } else {
                rank.to_string()
            };

            let num_rowers: u8 = self.get_column("BoatClass_NumRowers");
            let points: u8 = if rank > 0 { num_rowers + (5 - rank) } else { 0 };

            Some(HeatResult {
                delta,
                rank_label,
                rank_sort,
                net_time: self.get_column("Result_NetTime"),
                result: self.get_column("Result_DisplayValue"),
                points,
            })
        } else {
            None
        }
    }
}
