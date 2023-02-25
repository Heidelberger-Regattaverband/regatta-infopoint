use super::ToEntity;
use crate::db::tiberius::RowColumn;
use serde::Serialize;
use std::time::Duration;
use tiberius::Row;

#[derive(Debug, Serialize, Clone)]
pub struct HeatResult {
    #[serde(rename = "rankSort")]
    rank_sort: u8,
    #[serde(rename = "rankLabel")]
    rank_label: String,
    result: String,
    delta: String,
    points: u8,
}

impl ToEntity<HeatResult> for Row {
    fn to_entity(&self) -> HeatResult {
        let rank: u8 = self.get_column("Result_Rank");
        let rank_sort: u8 = if rank == 0 { u8::MAX } else { rank };
        let delta: String = if rank > 0 {
            let delta: i32 = self.get_column("Result_Delta");
            let duration = Duration::from_millis(delta as u64);
            let seconds = duration.as_secs();
            let millis = duration.subsec_millis() / 10;
            format!("{seconds}.{millis}")
        } else {
            Default::default()
        };

        let rank_label: String = if rank == 0 {
            Default::default()
        } else {
            rank.to_string()
        };

        let num_rowers: u8 = self.get_column("BoatClass_NumRowers");
        let points: u8 = if rank > 0 { num_rowers + (5 - rank) } else { 0 };

        HeatResult {
            delta,
            rank_label,
            rank_sort,
            result: self.get_column("Result_DisplayValue"),
            points,
        }
    }
}
