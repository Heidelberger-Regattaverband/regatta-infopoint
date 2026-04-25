use super::TryToEntity;
use super::boat_class::NUM_ROWERS;
use crate::tiberius::{RowColumn, TryRowColumn};
use ::serde::Serialize;
use ::tiberius::Row;
use ::utoipa::ToSchema;

const RANK: &str = "Result_Rank";
const DELTA: &str = "Result_Delta";
const DISPLAY_VALUE: &str = "Result_DisplayValue";
const NET_TIME: &str = "Result_NetTime";

#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HeatResult {
    /// The rank which can be used for sorting, e.g. DNS or DNF is rank 99
    pub rank_sort: u8,

    /// The rank label which can be different from the rank_sort.
    rank_label: String,
    result: String,

    /// The net time of the boat
    pub net_time: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,

    /// The points given for the result
    points: u8,
}

impl HeatResult {
    pub(crate) fn select_columns(alias: &str) -> String {
        format!(" {alias}.{RANK}, {alias}.{DELTA}, {alias}.{DISPLAY_VALUE}, {alias}.{NET_TIME} ")
    }
}

impl TryToEntity<HeatResult> for Row {
    fn try_to_entity(&self) -> Option<HeatResult> {
        if let Some(rank) = <Row as TryRowColumn<u8>>::try_get_column(self, RANK) {
            let num_rowers: u8 = self.get_column(NUM_ROWERS);
            let points: u8 = if rank > 0 && rank <= 5 {
                num_rowers + (5 - rank)
            } else {
                0
            };

            Some(HeatResult {
                delta: None,
                rank_label: if rank == 0 {
                    Default::default()
                } else {
                    rank.to_string()
                },
                rank_sort: if rank == 0 { u8::MAX } else { rank },
                net_time: self.get_column(NET_TIME),
                result: self.get_column(DISPLAY_VALUE),
                points,
            })
        } else {
            None
        }
    }
}
