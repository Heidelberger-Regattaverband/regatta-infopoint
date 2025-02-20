use crate::db::{
    model::{AgeClass, BoatClass, HeatRegistration, Race, Referee, TryToEntity, utils},
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use chrono::{DateTime, Utc};
use futures::future::join;
use serde::Serialize;
use tiberius::{Query, Row};

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

    state: u8,

    /// Indicates whether or not the heat has been canceled.
    cancelled: bool,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    referees: Vec<Referee>,

    #[serde(skip_serializing_if = "Option::is_none")]
    date_time: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) registrations: Option<Vec<HeatRegistration>>,

    /// The round of this heat: 64 - final, 4 - Vorlauf
    pub(crate) round: i16,
}

impl Heat {
    pub(crate) fn select_columns(alias: &str) -> String {
        format!(
            " {0}.Comp_ID, {0}.Comp_Number, {0}.Comp_RoundCode, {0}.Comp_Label, {0}.Comp_GroupValue, \
            {0}.Comp_State, {0}.Comp_Cancelled, {0}.Comp_DateTime, {0}.Comp_Round ",
            alias
        )
    }

    /// Query all heats of a regatta. The heats are ordered by their date and time.
    ///
    /// # Arguments
    /// * `regatta_id` - The regatta identifier
    /// * `pool` - The database connection pool
    /// # Returns
    /// A list of heats
    pub async fn query_heats_of_regatta(regatta_id: i32, pool: &TiberiusPool) -> Vec<Heat> {
        let sql = format!(
            "SELECT {0}, {1}, {2}, o.* FROM Comp c
            JOIN Offer o     ON o.Offer_ID              = c.Comp_Race_ID_FK
            JOIN AgeClass a  ON o.Offer_AgeClass_ID_FK  = a.AgeClass_ID
            JOIN BoatClass b ON o.Offer_BoatClass_ID_FK = b.BoatClass_ID
            WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_DateTime IS NOT NULL
            ORDER BY c.Comp_DateTime ASC",
            Heat::select_columns("c"),
            AgeClass::select_columns("a"),
            BoatClass::select_columns("b")
        );

        let mut query = Query::new(sql);
        query.bind(regatta_id);

        let mut client = pool.get().await;
        let heats = utils::get_rows(query.query(&mut client).await.unwrap()).await;
        heats.into_iter().map(|row| Heat::from(&row)).collect()
    }

    /// Query all heats of a race. The heats are ordered by their number.
    ///
    /// # Arguments
    /// * `race_id` - The race identifier
    /// * `pool` - The database connection pool
    /// # Returns
    /// A list of heats
    pub async fn query_heats_of_race(race_id: i32, pool: &TiberiusPool) -> Vec<Heat> {
        let sql = format!(
            "SELECT {0} FROM Comp c
            WHERE c.Comp_Race_ID_FK = @P1 AND c.Comp_DateTime IS NOT NULL
            ORDER BY c.Comp_Number ASC",
            Heat::select_columns("c")
        );
        let mut query = Query::new(sql);
        query.bind(race_id);

        let mut client = pool.get().await;
        let heats = utils::get_rows(query.query(&mut client).await.unwrap()).await;
        heats.into_iter().map(|row| Heat::from(&row)).collect()
    }

    /// Query a single heat.
    /// # Arguments
    /// * `heat_id` - The heat identifier
    /// * `pool` - The database connection pool
    /// # Returns
    /// The heat with the given identifier
    pub async fn query_single(heat_id: i32, pool: &TiberiusPool) -> Heat {
        let sql = format!(
            "SELECT {0}, {1}, {2}, o.* FROM Comp c
            JOIN Offer o     ON o.Offer_ID              = c.Comp_Race_ID_FK
            JOIN AgeClass a  ON o.Offer_AgeClass_ID_FK  = a.AgeClass_ID
            JOIN BoatClass b ON o.Offer_BoatClass_ID_FK = b.BoatClass_ID
            WHERE Comp_ID = @P1",
            Heat::select_columns("c"),
            AgeClass::select_columns("a"),
            BoatClass::select_columns("b")
        );

        let mut query = Query::new(sql);
        query.bind(heat_id);

        let mut client = pool.get().await;
        let mut heat = Heat::from(&utils::get_row(query.query(&mut client).await.unwrap()).await);

        let results = join(
            Referee::query_referees_for_heat(heat.id, pool),
            HeatRegistration::query_registrations_of_heat(&heat, pool),
        )
        .await;
        heat.referees = results.0;
        heat.registrations = Some(results.1);
        heat
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
            registrations: None,
            round: value.get_column("Comp_Round"),
        }
    }
}

impl TryToEntity<Heat> for Row {
    fn try_to_entity(&self) -> Option<Heat> {
        <Row as TryRowColumn<i32>>::try_get_column(self, "Comp_ID").map(|_id| Heat::from(self))
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Kiosk {
    pub finished: Vec<Heat>,
    pub running: Vec<Heat>,
    pub next: Vec<Heat>,
}
impl Kiosk {
    pub async fn query_finished(regatta_id: i32, pool: &TiberiusPool) -> Vec<Heat> {
        let mut query = Query::new("SELECT DISTINCT TOP 5 c.*, ac.*, o.Offer_GroupMode,
            o.Offer_HRV_Seeded, o.Offer_RaceNumber, o.Offer_ID, o.Offer_ShortLabel, o.Offer_LongLabel, o.Offer_Comment, o.Offer_Distance, o.Offer_IsLightweight, o.Offer_Cancelled
            FROM Comp AS c
            FULL OUTER JOIN Offer AS o     ON o.Offer_ID = c.Comp_Race_ID_FK
            FULL OUTER JOIN AgeClass AS ac ON o.Offer_AgeClass_ID_FK = ac.AgeClass_ID
            WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_State = 4 ORDER BY c.Comp_DateTime DESC");
        query.bind(regatta_id);
        let mut client = pool.get().await;
        let stream = query.query(&mut client).await.unwrap();
        let heats = utils::get_rows(stream).await;
        heats.into_iter().map(|row| Heat::from(&row)).collect()
    }

    pub async fn query_next(regatta_id: i32, pool: &TiberiusPool) -> Vec<Heat> {
        let mut query = Query::new("SELECT DISTINCT TOP 5 c.*, ac.*, o.Offer_GroupMode,
            o.Offer_HRV_Seeded, o.Offer_RaceNumber, o.Offer_ID, o.Offer_ShortLabel, o.Offer_LongLabel, o.Offer_Comment, o.Offer_Distance, o.Offer_IsLightweight, o.Offer_Cancelled
            FROM Comp AS c
            FULL OUTER JOIN Offer AS o     ON o.Offer_ID = c.Comp_Race_ID_FK
            FULL OUTER JOIN AgeClass AS ac ON o.Offer_AgeClass_ID_FK = ac.AgeClass_ID
            WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_State = 1 AND c.Comp_Cancelled = 0 ORDER BY c.Comp_DateTime ASC");
        query.bind(regatta_id);
        let mut client = pool.get().await;
        let stream = query.query(&mut client).await.unwrap();
        let heats = utils::get_rows(stream).await;
        heats.into_iter().map(|row| Heat::from(&row)).collect()
    }
}
