use crate::{
    aquarius::model::{AgeClass, BoatClass, HeatEntry, Race, Referee, TryToEntity, utils},
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use chrono::{DateTime, Utc};
use futures::future::join;
use serde::Serialize;
use tiberius::{Query, Row, error::Error as DbError};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Heat {
    /// The unique identifier of this heat.
    pub id: i32,

    /// The sequential number of the heat.
    number: i16,

    /// The race the heat belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    race: Option<Race>,

    /// The round code of the heat. Known values are: "R" - main race, "A" - division, "V" - Vorlauf
    round_code: String,

    /// An optional division label, e.g. "1" or "2"
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,

    group_value: i16,

    /// The state of the heat. Known values are:
    /// 0 - scheduled, 1 - seeded, 2 - started, 3 - ???, 4 - official, 5 - finished, 6 - photo finish
    state: u8,

    /// Indicates whether or not the heat has been canceled.
    cancelled: bool,

    /// The umpires of this heat.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    referees: Vec<Referee>,

    /// The date and time of the heat.
    #[serde(skip_serializing_if = "Option::is_none")]
    date_time: Option<DateTime<Utc>>,

    /// The entries assigned to this heat.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) entries: Option<Vec<HeatEntry>>,

    /// The round of this heat: 64 - final, 4 - Vorlauf
    pub(crate) round: i16,
}

impl Heat {
    pub(crate) fn select_columns(alias: &str) -> String {
        format!(
            " {alias}.Comp_ID, {alias}.Comp_Number, {alias}.Comp_RoundCode, {alias}.Comp_Label, {alias}.Comp_GroupValue, \
            {alias}.Comp_State, {alias}.Comp_Cancelled, {alias}.Comp_DateTime, {alias}.Comp_Round "
        )
    }

    /// Query all heats of a regatta. The heats are ordered by their date and time.
    ///
    /// # Arguments
    /// * `regatta_id` - The regatta identifier
    /// * `pool` - The database connection pool
    /// # Returns
    /// A list of heats
    pub async fn query_heats_of_regatta(regatta_id: i32, pool: &TiberiusPool) -> Result<Vec<Self>, DbError> {
        let sql = format!(
            "SELECT {0}, {1}, {2}, o.* FROM Comp c
            JOIN Offer     o ON o.Offer_ID              = c.Comp_Race_ID_FK
            JOIN AgeClass  a ON o.Offer_AgeClass_ID_FK  = a.AgeClass_ID
            JOIN BoatClass b ON o.Offer_BoatClass_ID_FK = b.BoatClass_ID
            WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_DateTime IS NOT NULL
            ORDER BY c.Comp_DateTime ASC",
            Heat::select_columns("c"),
            AgeClass::select_minimal_columns("a"),
            BoatClass::select_columns("b")
        );

        let mut query = Query::new(sql);
        query.bind(regatta_id);

        let mut client = pool.get().await;
        let heats = utils::get_rows(query.query(&mut client).await?).await?;
        Ok(heats.into_iter().map(|row| Heat::from(&row)).collect())
    }

    /// Query all heats of a race. The heats are ordered by their number.
    ///
    /// # Arguments
    /// * `race_id` - The race identifier
    /// * `pool` - The database connection pool
    /// # Returns
    /// A list of heats
    pub async fn query_heats_of_race(race_id: i32, pool: &TiberiusPool) -> Result<Vec<Self>, DbError> {
        let sql = format!(
            "SELECT {0} FROM Comp c
            WHERE c.Comp_Race_ID_FK = @P1 AND c.Comp_DateTime IS NOT NULL
            ORDER BY c.Comp_Number ASC",
            Heat::select_columns("c")
        );
        let mut query = Query::new(sql);
        query.bind(race_id);

        let mut client = pool.get().await;
        let heats = utils::get_rows(query.query(&mut client).await?).await?;
        Ok(heats.into_iter().map(|row| Heat::from(&row)).collect())
    }

    /// Query all heats of an entry.
    ///
    /// # Arguments
    /// * `entry_id` - The entry identifier
    /// * `pool` - The database connection pool
    /// # Returns
    /// A list of heats
    pub async fn query_heats_of_entry(entry_id: i32, pool: &TiberiusPool) -> Result<Vec<Self>, DbError> {
        let sql = format!(
            "SELECT {0} FROM Comp c
            JOIN CompEntries ce ON c.Comp_ID  = ce.CE_Comp_ID_FK
            JOIN Entry        e ON e.Entry_ID = ce.CE_Entry_ID_FK
            WHERE e.Entry_ID = @P1
            ORDER BY c.Comp_Round ASC",
            Heat::select_columns("c")
        );
        let mut query = Query::new(sql);
        query.bind(entry_id);
        let mut client = pool.get().await;
        let heats = utils::get_rows(query.query(&mut client).await?).await?;
        Ok(heats.into_iter().map(|row| Heat::from(&row)).collect())
    }

    /// Query a single heat.
    /// # Arguments
    /// * `heat_id` - The heat identifier
    /// * `pool` - The database connection pool
    /// # Returns
    /// The heat with the given identifier
    pub async fn query_single(heat_id: i32, pool: &TiberiusPool) -> Result<Self, DbError> {
        let sql = format!(
            "SELECT {0}, {1}, {2}, o.* FROM Comp c
            JOIN Offer o     ON o.Offer_ID              = c.Comp_Race_ID_FK
            JOIN AgeClass a  ON o.Offer_AgeClass_ID_FK  = a.AgeClass_ID
            JOIN BoatClass b ON o.Offer_BoatClass_ID_FK = b.BoatClass_ID
            WHERE Comp_ID = @P1",
            Heat::select_columns("c"),
            AgeClass::select_minimal_columns("a"),
            BoatClass::select_columns("b")
        );

        let mut query = Query::new(sql);
        query.bind(heat_id);

        let mut client = pool.get().await;
        let mut heat = Heat::from(&utils::get_row(query.query(&mut client).await?).await?);

        let results = join(
            Referee::query_referees_for_heat(heat.id, pool),
            HeatEntry::query_entries_of_heat(&heat, pool),
        )
        .await;
        heat.referees = results.0?;
        heat.entries = Some(results.1?);
        Ok(heat)
    }
}

impl From<&Row> for Heat {
    fn from(value: &Row) -> Self {
        Heat {
            id: value.get_column("Comp_ID"),
            race: value.try_to_entity(),
            number: value.get_column("Comp_Number"),
            round_code: value.get_column("Comp_RoundCode"),
            label: value.try_get_column("Comp_Label"),
            group_value: value.get_column("Comp_GroupValue"),
            state: value.get_column("Comp_State"),
            cancelled: value.get_column("Comp_Cancelled"),
            date_time: value.try_get_column("Comp_DateTime"),
            referees: vec![],
            entries: None,
            round: value.get_column("Comp_Round"),
        }
    }
}

impl TryToEntity<Heat> for Row {
    fn try_to_entity(&self) -> Option<Heat> {
        <Row as TryRowColumn<i32>>::try_get_column(self, "Comp_ID").map(|_id| Heat::from(self))
    }
}
