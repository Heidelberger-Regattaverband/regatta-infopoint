use crate::db::{
    model::utils,
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Serialize;
use tiberius::{Query, Row, error::Error as DbError};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    /// The date and time when the schedule was generated
    generated: DateTime<Utc>,

    /// The schedule entries
    entries: Vec<ScheduleEntry>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleEntry {
    /// The race number
    race_number: String,

    /// The race short label (e.g. "MM 2x A-K")
    race_short_label: String,

    /// The number of boats in the race
    boats: i32,

    /// distance in meters
    distance: i16,

    /// The number of heats in the finals
    final_heats: i32,

    /// The number of heats in the forerun
    forerun_heats: i32,

    /// The date and time when the finals start
    #[serde(skip_serializing_if = "Option::is_none")]
    final_start: Option<DateTime<Utc>>,

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
            final_heats: row.get_column("Final_Heats"),
            forerun_heats: row.get_column("Forerun_Heats"),
            final_start: row.try_get_column("Final_Start").map(|f: NaiveDateTime| f.and_utc()),
            forerun_start: row.try_get_column("Forerun_Start").map(|f: NaiveDateTime| f.and_utc()),
        }
    }
}

impl Schedule {
    pub async fn query_schedule_for_regatta(regatta_id: i32, pool: &TiberiusPool) -> Result<Self, DbError> {
        let sql = "SELECT o.Offer_RaceNumber, o.Offer_ShortLabel, o.Offer_Distance,
            (SELECT Count(*) FROM Entry e WHERE e.Entry_Race_ID_FK = o.Offer_ID AND e.Entry_CancelValue = 0) as Boats,
            (SELECT Count(*) FROM Comp c WHERE c.Comp_Race_ID_FK = o.Offer_ID AND c.Comp_Cancelled = 0 
                AND c.Comp_RoundCode IN ('R', 'A', 'F')) as Final_Heats,
            (SELECT Count(*) FROM Comp c WHERE c.Comp_Race_ID_FK = o.Offer_ID AND c.Comp_Cancelled = 0 
                AND c.Comp_RoundCode IN ('V')) as Forerun_Heats,
            (SELECT MIN(Comp_DateTime) FROM Comp c WHERE c.Comp_Race_ID_FK = o.Offer_ID AND c.Comp_Cancelled = 0 
                AND c.Comp_RoundCode IN ('R', 'A', 'F')) as Final_Start,
            (SELECT MIN(Comp_DateTime) FROM Comp c WHERE c.Comp_Race_ID_FK = o.Offer_ID AND c.Comp_Cancelled = 0 
                AND c.Comp_RoundCode IN ('V')) as Forerun_Start
            FROM Offer o
            WHERE o.Offer_Event_ID_FK = @P1 AND o.Offer_Cancelled = 0 
            ORDER BY Final_Start";

        let mut query: Query = Query::new(sql);
        query.bind(regatta_id);

        let mut client = pool.get().await;
        let stream = query.query(&mut client).await?;
        let entries: Vec<ScheduleEntry> = utils::get_rows(stream)
            .await?
            .into_iter()
            .map(|row| ScheduleEntry::from(&row))
            .filter(|entry| entry.final_heats > 0 || entry.forerun_heats > 0)
            .collect();
        Ok(Schedule {
            generated: Utc::now(),
            entries,
        })
    }
}
