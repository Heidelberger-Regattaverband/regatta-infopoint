use crate::db::{
    model::utils,
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Schedule {
    /// The date and time when the schedule was generated
    generated: DateTime<Utc>,

    /// The schedule entries
    entries: Vec<ScheduleEntry>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScheduleEntry {
    /// The race number
    race_number: String,

    /// The race short label (e.g. "MM 2x A-K")
    race_short_label: String,

    /// The number of boats in the race
    boats: i32,

    /// distance in meters
    distance: i16,

    /// The number of heats in the finals
    finals_heats: i32,

    /// The number of heats in the forerun
    forerun_heats: i32,

    /// The date and time when the finals start
    #[serde(skip_serializing_if = "Option::is_none")]
    finals_start: Option<DateTime<Utc>>,

    /// The date and time when the forerun starts
    #[serde(skip_serializing_if = "Option::is_none")]
    forerun_start: Option<DateTime<Utc>>,
}

impl From<&Row> for ScheduleEntry {
    fn from(row: &Row) -> Self {
        ScheduleEntry {
            race_number: row.get_column("Offer_RaceNumber"),
            distance: row.get_column("Offer_Distance"),
            boats: row.get_column("Boats"),
            race_short_label: row.get_column("Offer_ShortLabel"),
            finals_heats: row.get_column("Heats_Finals"),
            forerun_heats: row.get_column("Heats_Forerun"),
            finals_start: row.try_get_column("Start_Finals").map(|f: NaiveDateTime| f.and_utc()),
            forerun_start: row.try_get_column("Start_Forerun").map(|f: NaiveDateTime| f.and_utc()),
        }
    }
}

impl Schedule {
    pub(crate) async fn query_schedule_for_regatta(regatta_id: i32, pool: &TiberiusPool) -> Self {
        let sql = "SELECT o.Offer_RaceNumber, o.Offer_ShortLabel, o.Offer_Distance,
            (SELECT Count(*) FROM Entry e WHERE e.Entry_Race_ID_FK = o.Offer_ID AND e.Entry_CancelValue = 0) as Boats,
            (SELECT Count(*) FROM Comp c WHERE c.Comp_Race_ID_FK = o.Offer_ID AND c.Comp_Cancelled = 0 
                AND c.Comp_RoundCode IN ('R', 'A', 'F')) as Heats_Finals,
            (SELECT Count(*) FROM Comp c WHERE c.Comp_Race_ID_FK = o.Offer_ID AND c.Comp_Cancelled = 0 
                AND c.Comp_RoundCode IN ('V')) as Heats_Forerun,
            (SELECT MIN(Comp_DateTime) FROM Comp c WHERE c.Comp_Race_ID_FK = o.Offer_ID AND c.Comp_Cancelled = 0 
                AND c.Comp_RoundCode IN ('R', 'A', 'F')) as Start_Finals,
            (SELECT MIN(Comp_DateTime) FROM Comp c WHERE c.Comp_Race_ID_FK = o.Offer_ID AND c.Comp_Cancelled = 0 
                AND c.Comp_RoundCode IN ('V')) as Start_Forerun
            FROM Offer o
            WHERE o.Offer_Event_ID_FK = @P1 AND o.Offer_Cancelled = 0
            ORDER BY o.Offer_SortValue";

        let mut query: Query = Query::new(sql);
        query.bind(regatta_id);

        let mut client = pool.get().await;
        let stream = query.query(&mut client).await.unwrap();
        let entries: Vec<ScheduleEntry> = utils::get_rows(stream)
            .await
            .into_iter()
            .map(|row| ScheduleEntry::from(&row))
            .collect();
        Schedule {
            generated: Utc::now(),
            entries,
        }
    }
}
