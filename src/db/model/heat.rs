use crate::db::{
    aquarius::AquariusClient,
    model::{utils, HeatRegistration, Race, Referee, ToEntity, TryToEntity},
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use chrono::{DateTime, Utc};
use futures::future::join;
use serde::Serialize;
use tiberius::{Query, Row};

use super::{AgeClass, BoatClass};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Heat {
    /// The unique identifier of this heat.
    pub id: i32,

    /// The sequential number of the heat.
    number: i16,

    /// The race the heat belongs to.
    race: Race,

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
    pub registrations: Option<Vec<HeatRegistration>>,

    /// The round of this heat: 64 - final, 4 - Vorlauf
    pub round: i16,
}

impl Heat {
    pub fn select_columns(alias: &str) -> String {
        format!(" {0}.Comp_ID, {0}.Comp_Number, {0}.Comp_RoundCode, {0}.Comp_Label, {0}.Comp_GroupValue, {0}.Comp_State, {0}.Comp_Cancelled, {0}.Comp_DateTime, {0}.Comp_Round ", alias)
    }
}

impl ToEntity<Heat> for Row {
    fn to_entity(&self) -> Heat {
        Heat {
            id: self.get_column("Comp_ID"),
            race: self.to_entity(),
            number: self.get_column("Comp_Number"),
            round_code: self.get_column("Comp_RoundCode"),
            label: self.try_get_column("Comp_Label"),
            group_value: self.get_column("Comp_GroupValue"),
            state: self.get_column("Comp_State"),
            cancelled: self.get_column("Comp_Cancelled"),
            date_time: self.try_get_column("Comp_DateTime"),
            referees: vec![],
            registrations: None,
            round: self.get_column("Comp_Round"),
        }
    }
}

impl TryToEntity<Heat> for Row {
    fn try_to_entity(&self) -> Option<Heat> {
        <Row as TryRowColumn<i32>>::try_get_column(self, "Comp_ID").map(|_id| self.to_entity())
    }
}

impl Heat {
    pub async fn query_all(regatta_id: i32, pool: &TiberiusPool) -> Vec<Heat> {
        let mut client = pool.get().await;
        let mut query = Query::new(
            "SELECT DISTINCT".to_string()
                + &Heat::select_columns("c")
                + ","
                + &AgeClass::select_columns("a")
                + ","
                + &BoatClass::select_columns("b")
                + ", o.*
            FROM Comp c
            JOIN Offer o     ON o.Offer_ID              = c.Comp_Race_ID_FK
            JOIN AgeClass a  ON o.Offer_AgeClass_ID_FK  = a.AgeClass_ID
            JOIN BoatClass b ON o.Offer_BoatClass_ID_FK = b.BoatClass_ID
            WHERE Comp_Event_ID_FK = @P1 ORDER BY Comp_DateTime ASC",
        );
        query.bind(regatta_id);
        let stream = query.query(&mut client).await.unwrap();
        let heats = utils::get_rows(stream).await;
        heats.into_iter().map(|row| row.to_entity()).collect()
    }

    pub async fn query_single(heat_id: i32, pool: &TiberiusPool) -> Heat {
        let mut client = pool.get().await;
        let mut query = Query::new(
            "SELECT DISTINCT".to_string()
                + &Heat::select_columns("c")
                + ","
                + &AgeClass::select_columns("a")
                + ","
                + &BoatClass::select_columns("b")
                + ", o.*
            FROM Comp c
            JOIN Offer o     ON o.Offer_ID              = c.Comp_Race_ID_FK
            JOIN AgeClass a  ON o.Offer_AgeClass_ID_FK  = a.AgeClass_ID
            JOIN BoatClass b ON o.Offer_BoatClass_ID_FK = b.BoatClass_ID
            WHERE Comp_ID = @P1",
        );
        query.bind(heat_id);
        let stream = query.query(&mut client).await.unwrap();
        let mut heat: Heat = utils::get_row(stream).await.to_entity();

        let result = join(Referee::query(heat.id, pool), HeatRegistration::query_all(&heat, pool)).await;
        heat.referees = result.0;
        heat.registrations = Some(result.1);

        heat
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Kiosk {
    pub finished: Vec<Heat>,
    pub running: Vec<Heat>,
    pub next: Vec<Heat>,
}
impl Kiosk {
    pub async fn query_finished(regatta_id: i32, client: &mut AquariusClient<'_>) -> Vec<Heat> {
        let mut query = Query::new("SELECT DISTINCT TOP 5 c.*, ac.*, o.Offer_GroupMode,
            o.Offer_HRV_Seeded, o.Offer_RaceNumber, o.Offer_ID, o.Offer_ShortLabel, o.Offer_LongLabel, o.Offer_Comment, o.Offer_Distance, o.Offer_IsLightweight, o.Offer_Cancelled
            FROM Comp AS c
            FULL OUTER JOIN Offer AS o     ON o.Offer_ID = c.Comp_Race_ID_FK
            FULL OUTER JOIN AgeClass AS ac ON o.Offer_AgeClass_ID_FK = ac.AgeClass_ID
            WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_State = 4 ORDER BY c.Comp_DateTime DESC");
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let heats = utils::get_rows(stream).await;
        heats.into_iter().map(|row| row.to_entity()).collect()
    }

    pub async fn query_next(regatta_id: i32, client: &mut AquariusClient<'_>) -> Vec<Heat> {
        let mut query = Query::new("SELECT DISTINCT TOP 5 c.*, ac.*, o.Offer_GroupMode,
            o.Offer_HRV_Seeded, o.Offer_RaceNumber, o.Offer_ID, o.Offer_ShortLabel, o.Offer_LongLabel, o.Offer_Comment, o.Offer_Distance, o.Offer_IsLightweight, o.Offer_Cancelled
            FROM Comp AS c
            FULL OUTER JOIN Offer AS o     ON o.Offer_ID = c.Comp_Race_ID_FK
            FULL OUTER JOIN AgeClass AS ac ON o.Offer_AgeClass_ID_FK = ac.AgeClass_ID
            WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_State = 1 AND c.Comp_Cancelled = 0 ORDER BY c.Comp_DateTime ASC");
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let heats = utils::get_rows(stream).await;
        heats.into_iter().map(|row| row.to_entity()).collect()
    }
}
