use super::age_class::ID as AGE_CLASS_ID;
use super::boat_class::ID as BOAT_CLASS_ID;
use super::entry::CANCELLED as ENTRY_CANCELLED;
use super::get_row;
use super::get_rows;
use super::heat::CANCELLED as HEAT_CANCELLED;
use super::heat::DATE_TIME as HEAT_DATE_TIME;
use crate::tiberius::TiberiusClient;
use crate::{
    aquarius::model::{AgeClass, BoatClass, Entry, Heat, TryToEntity},
    error::DbError,
    tiberius::{RowColumn, TryRowColumn},
};
use ::chrono::{DateTime, Utc};
use ::serde::Serialize;
use ::tiberius::{Query, Row};
use ::utoipa::ToSchema;

pub(super) const ID: &str = "Offer_ID";
const RACE_NUMBER: &str = "Offer_RaceNumber";
const SHORT_LABEL: &str = "Offer_ShortLabel";
const LONG_LABEL: &str = "Offer_LongLabel";
const COMMENT: &str = "Offer_Comment";
const DISTANCE: &str = "Offer_Distance";
const IS_LIGHTWEIGHT: &str = "Offer_IsLightweight";
pub(super) const CANCELLED: &str = "Offer_Cancelled";
const DRIVEN: &str = "Offer_Driven";
const GROUP_MODE: &str = "Offer_GroupMode";
const SORT_VALUE: &str = "Offer_SortValue";
const HRV_SEEDED: &str = "Offer_HRV_Seeded";
const ENTRIES_COUNT: &str = "Entries_Count";
const HEATS_COUNT: &str = "Heats_Count";
const RACE_STATE: &str = "Race_State";
const RACE_DATE_TIME: &str = "Race_DateTime";

#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Race {
    /// The unique identifier of the race.
    pub id: i32,

    /// The race number, e.g. "101", "115a", ...
    number: String,

    /// The short label of the race, e.g. "OFF 2x", "MM 4x", ...
    short_label: String,

    /// The long label of the race, e.g. "Offene Klasse-Doppelzweier", "Masters-Männer-Doppelvierer", ...
    long_label: String,

    /// An optional comment, e.g. "A-K" or "Lgr. III"
    comment: String,

    /// The distance of the race, e.g. 1000, 1500 or 350
    distance: i16,

    /// Indicates whether the race is a lightweight or not.
    lightweight: bool,

    /// The state of the race, e.g. 0 = not started, 4 = finished
    state: i32,

    /// Indicates whether the race is canceled or not.
    cancelled: bool,

    /// The number of entries for this race.
    entries_count: i32,

    /// Indicates whether the race is seeded or not.
    seeded: bool,

    /// The age class of this race.
    #[serde(skip_serializing_if = "Option::is_none")]
    age_class: Option<AgeClass>,

    /// The boat class of this race.
    #[serde(skip_serializing_if = "Option::is_none")]
    boat_class: Option<BoatClass>,

    /// The group mode of this race. Known values: 2 = Masters / Age Classes
    group_mode: u8,

    /// The date and time of the first heat of this race.
    #[serde(skip_serializing_if = "Option::is_none")]
    date_time: Option<DateTime<Utc>>,

    /// All entries for this race.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<Entry>>,

    /// All heats for this race.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heats: Option<Vec<Heat>>,

    /// The number of heats for this race.
    heats_count: i32,
}

impl From<&Row> for Race {
    fn from(row: &Row) -> Self {
        let short_label: String = row.get_column(SHORT_LABEL);
        let long_label: String = row.get_column(LONG_LABEL);
        let comment: String = row.try_get_column(COMMENT).unwrap_or_default();
        let seeded: Option<bool> = row.try_get_column(HRV_SEEDED);
        let cancelled = row.get_column(CANCELLED);
        let driven: bool = row.get_column(DRIVEN);
        Race {
            id: row.get_column(ID),
            comment: comment.trim().to_owned(),
            number: row.get_column(RACE_NUMBER),
            short_label: short_label.trim().to_owned(),
            long_label: long_label.trim().to_owned(),
            distance: row.get_column(DISTANCE),
            lightweight: row.get_column(IS_LIGHTWEIGHT),
            cancelled: cancelled || !driven,
            entries_count: row.try_get_column(ENTRIES_COUNT).unwrap_or_default(),
            heats_count: row.get_column(HEATS_COUNT),
            seeded: seeded.unwrap_or_default(),
            age_class: row.try_to_entity(),
            boat_class: row.try_to_entity(),
            state: row.try_get_column(RACE_STATE).unwrap_or_default(),
            group_mode: row.get_column(GROUP_MODE),
            date_time: row.try_get_column(RACE_DATE_TIME),
            entries: None,
            heats: None,
        }
    }
}

impl TryToEntity<Race> for Row {
    fn try_to_entity(&self) -> Option<Race> {
        <Row as TryRowColumn<i32>>::try_get_column(self, ID).map(|_id| Race::from(self))
    }
}

impl Race {
    pub(crate) fn select_columns(alias: &str) -> String {
        format!(
            "{alias}.{ID}, {alias}.{RACE_NUMBER}, {alias}.{DISTANCE}, {alias}.{IS_LIGHTWEIGHT}, {alias}.{CANCELLED}, {alias}.{DRIVEN}, {alias}.{SHORT_LABEL}, \
            {alias}.{LONG_LABEL}, {alias}.{COMMENT}, {alias}.{GROUP_MODE}, {alias}.{SORT_VALUE}, {alias}.{HRV_SEEDED}, \
            (SELECT Count(*) FROM Comp c WHERE c.Comp_Race_ID_FK = {alias}.{ID} AND c.{HEAT_CANCELLED} = 0) as {HEATS_COUNT}"
        )
    }
    pub(crate) fn select_columns_with_analytical(alias: &str) -> String {
        format!(
            "{alias}.{ID}, {alias}.{RACE_NUMBER}, {alias}.{DISTANCE}, {alias}.{IS_LIGHTWEIGHT}, {alias}.{CANCELLED}, {alias}.{DRIVEN}, {alias}.{SHORT_LABEL}, \
            {alias}.{LONG_LABEL}, {alias}.{COMMENT}, {alias}.{GROUP_MODE}, {alias}.{SORT_VALUE}, {alias}.{HRV_SEEDED}, \
            (SELECT Count(*) FROM Entry e WHERE e.Entry_Race_ID_FK = {alias}.{ID} AND e.{ENTRY_CANCELLED} = 0) as {ENTRIES_COUNT}, \
            (SELECT Count(*) FROM Comp  c WHERE c.Comp_Race_ID_FK = {alias}.{ID} AND c.{HEAT_CANCELLED} = 0) as {HEATS_COUNT}, \
            (SELECT AVG(Comp_State) FROM Comp c WHERE c.Comp_Race_ID_FK = {alias}.{ID} AND c.{HEAT_DATE_TIME} IS NOT NULL AND c.{HEAT_CANCELLED} = 0) as {RACE_STATE}, \
            (SELECT MIN({HEAT_DATE_TIME}) FROM Comp c WHERE c.Comp_Race_ID_FK = {alias}.{ID} AND c.{HEAT_DATE_TIME} IS NOT NULL AND c.{HEAT_CANCELLED} = 0) as {RACE_DATE_TIME}"
        )
    }

    /// Query all races of a regatta.
    /// # Arguments
    /// * `regatta_id` - The regatta identifier
    /// * `pool` - The database connection pool
    /// # Returns
    /// A list with races of the regatta
    pub async fn query_races_of_regatta(regatta_id: i32, client: &mut TiberiusClient) -> Result<Vec<Self>, DbError> {
        let sql = format!(
            "SELECT {0}, {1}, {2} FROM Offer o
            JOIN AgeClass  a ON o.Offer_AgeClass_ID_FK  = a.{AGE_CLASS_ID}
            JOIN BoatClass b ON o.Offer_BoatClass_ID_FK = b.{BOAT_CLASS_ID}
            WHERE o.Offer_Event_ID_FK = @P1
            ORDER BY o.Offer_SortValue ASC",
            Race::select_columns_with_analytical("o"),
            AgeClass::select_minimal_columns("a"),
            BoatClass::select_columns("b")
        );
        let mut query = Query::new(sql);
        query.bind(regatta_id);

        let stream = query.query(client).await?;
        let races = get_rows(stream).await?;
        Ok(races.into_iter().map(|row| Race::from(&row)).collect())
    }

    pub async fn query_race_by_id(race_id: i32, client: &mut TiberiusClient) -> Result<Self, DbError> {
        let sql = format!(
            "SELECT {0}, {1}, {2} FROM Offer o
            JOIN AgeClass  a ON o.Offer_AgeClass_ID_FK  = a.{AGE_CLASS_ID}
            JOIN BoatClass b ON o.Offer_BoatClass_ID_FK = b.{BOAT_CLASS_ID}
            WHERE o.{ID} = @P1",
            Race::select_columns_with_analytical("o"),
            AgeClass::select_minimal_columns("a"),
            BoatClass::select_columns("b")
        );
        let mut query = Query::new(sql);
        query.bind(race_id);
        let stream = query.query(client).await?;
        Ok(Race::from(&get_row(stream).await?))
    }
}
