use crate::db::model::{Entry, Heat, utils};
use crate::db::{
    model::{AgeClass, BoatClass, TryToEntity},
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tiberius::{Query, Row, error::Error as DbError};

#[derive(Debug, Serialize, Clone)]
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
        let short_label: String = row.get_column("Offer_ShortLabel");
        let long_label: String = row.get_column("Offer_LongLabel");
        let comment: String = row.try_get_column("Offer_Comment").unwrap_or_default();
        let seeded: Option<bool> = row.try_get_column("Offer_HRV_Seeded");

        Race {
            id: row.get_column("Offer_ID"),
            comment: comment.trim().to_owned(),
            number: row.get_column("Offer_RaceNumber"),
            short_label: short_label.trim().to_owned(),
            long_label: long_label.trim().to_owned(),
            distance: row.get_column("Offer_Distance"),
            lightweight: row.get_column("Offer_IsLightweight"),
            cancelled: row.get_column("Offer_Cancelled"),
            entries_count: row.try_get_column("Entries_Count").unwrap_or_default(),
            heats_count: row.try_get_column("Heats_Count").unwrap_or_default(),
            seeded: seeded.unwrap_or_default(),
            age_class: row.try_to_entity(),
            boat_class: row.try_to_entity(),
            state: row.try_get_column("Race_State").unwrap_or_default(),
            group_mode: row.get_column("Offer_GroupMode"),
            date_time: row.try_get_column("Race_DateTime"),
            entries: None,
            heats: None,
        }
    }
}

impl TryToEntity<Race> for Row {
    fn try_to_entity(&self) -> Option<Race> {
        <Row as TryRowColumn<i32>>::try_get_column(self, "Offer_ID").map(|_id| Race::from(self))
    }
}

impl Race {
    pub(crate) fn select_columns(alias: &str) -> String {
        format!(
            " {0}.Offer_ID, {0}.Offer_RaceNumber, {0}.Offer_Distance, {0}.Offer_IsLightweight, {0}.Offer_Cancelled, {0}.Offer_ShortLabel, \
            {0}.Offer_LongLabel, {0}.Offer_Comment, {0}.Offer_GroupMode, {0}.Offer_SortValue, {0}.Offer_HRV_Seeded, \
            (SELECT Count(*) FROM Entry e WHERE e.Entry_Race_ID_FK = {0}.Offer_ID AND e.Entry_CancelValue = 0) as Entries_Count, \
            (SELECT Count(*) FROM Comp  c WHERE c.Comp_Race_ID_FK = {0}.Offer_ID AND c.Comp_Cancelled = 0) as Heats_Count, \
            (SELECT AVG(Comp_State) FROM Comp c WHERE c.Comp_Race_ID_FK = {0}.Offer_ID AND c.Comp_Cancelled = 0) as Race_State, \
            (SELECT MIN(Comp_DateTime) FROM Comp c WHERE c.Comp_Race_ID_FK = {0}.Offer_ID AND c.Comp_Cancelled = 0) as Race_DateTime \
        ",
            alias
        )
    }

    /// Query all races of a regatta.
    /// # Arguments
    /// * `regatta_id` - The regatta identifier
    /// * `pool` - The database connection pool
    /// # Returns
    /// A list with races of the regatta
    pub async fn query_races_of_regatta(regatta_id: i32, pool: &TiberiusPool) -> Result<Vec<Self>, DbError> {
        let sql = format!(
            "SELECT {0}, {1}, {2} FROM Offer o
            JOIN AgeClass  a ON o.Offer_AgeClass_ID_FK  = a.AgeClass_ID
            JOIN BoatClass b ON o.Offer_BoatClass_ID_FK = b.BoatClass_ID
            WHERE o.Offer_Event_ID_FK = @P1
            ORDER BY o.Offer_SortValue ASC",
            Race::select_columns("o"),
            AgeClass::select_minimal_columns("a"),
            BoatClass::select_columns("b")
        );
        let mut query = Query::new(sql);
        query.bind(regatta_id);

        let mut client = pool.get().await;
        let stream = query.query(&mut client).await?;
        let races = utils::get_rows(stream).await?;
        Ok(races.into_iter().map(|row| Race::from(&row)).collect())
    }

    pub async fn query_race_by_id(race_id: i32, pool: &TiberiusPool) -> Result<Self, DbError> {
        let sql = format!(
            "SELECT {0}, {1}, {2} FROM Offer o
            JOIN AgeClass  a ON o.Offer_AgeClass_ID_FK  = a.AgeClass_ID
            JOIN BoatClass b ON o.Offer_BoatClass_ID_FK = b.BoatClass_ID
            WHERE o.Offer_ID = @P1",
            Race::select_columns("o"),
            AgeClass::select_minimal_columns("a"),
            BoatClass::select_columns("b")
        );
        let mut client = pool.get().await;
        let mut query = Query::new(sql);
        query.bind(race_id);
        let stream = query.query(&mut client).await?;
        Ok(Race::from(&utils::get_row(stream).await?))
    }
}
