use crate::db::{
    aquarius::AquariusClient,
    model::{utils, Race, Referee, ToEntity, TryToEntity},
    tiberius::{RowColumn, TryRowColumn},
};
use chrono::{DateTime, Utc};
use log::info;
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Heat {
    pub id: i32,
    number: i16,
    round_code: String,
    label: String,
    group_value: i16,
    state: u8,
    cancelled: bool,
    race: Race,
    #[serde(skip_serializing_if = "Option::is_none")]
    referee: Option<Referee>,
    date_time: Option<DateTime<Utc>>,
}

impl ToEntity<Heat> for Row {
    fn to_entity(&self) -> Heat {
        let id: i32 = self.get_column("Comp_ID");
        let number: i16 = self.get_column("Comp_Number");
        let round_code: String = self.get_column("Comp_RoundCode");
        let label: String = self.get_column("Comp_Label");
        let group_value: i16 = self.get_column("Comp_GroupValue");
        let state: u8 = self.get_column("Comp_State");
        let cancelled: bool = self.get_column("Comp_Cancelled");

        Heat {
            id,
            race: self.to_entity(),
            number,
            round_code,
            label,
            group_value,
            state,
            cancelled,
            date_time: self.try_get_column("Comp_DateTime"),
            referee: self.try_to_entity(),
        }
    }
}

impl Heat {
    pub async fn query_all<'a>(regatta_id: i32, client: &mut AquariusClient<'_>) -> Vec<Heat> {
        let mut query = Query::new(
            "SELECT DISTINCT Comp.*, AgeClass.*, BoatClass.*, Referee.*, Offer.*
            FROM Comp
            FULL OUTER JOIN Offer       ON Offer_ID                  = Comp_Race_ID_FK
            FULL OUTER JOIN AgeClass    ON Offer_AgeClass_ID_FK      = AgeClass_ID
            JOIN BoatClass              ON Offer_BoatClass_ID_FK     = BoatClass_ID
            FULL OUTER JOIN CompReferee ON CompReferee_Comp_ID_FK    = Comp_ID
            FULL OUTER JOIN Referee     ON CompReferee_Referee_ID_FK = Referee_ID
            WHERE Comp_Event_ID_FK = @P1 ORDER BY Comp_DateTime ASC",
        );
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let heats = utils::get_rows(stream).await;
        heats.into_iter().map(|row| row.to_entity()).collect()
    }

    pub async fn search<'a>(regatta_id: i32, filter: String, client: &mut AquariusClient<'_>) -> Vec<Heat> {
        let sql = format!("SELECT DISTINCT c.*, ac.*, bc.*, r.*,
          o.Offer_HRV_Seeded, o.Offer_RaceNumber, o.Offer_ID, o.Offer_ShortLabel, o.Offer_LongLabel, o.Offer_Comment, o.Offer_Distance, o.Offer_IsLightweight, o.Offer_Cancelled
          FROM Comp AS c
          FULL OUTER JOIN Offer AS o ON o.Offer_ID = c.Comp_Race_ID_FK
          FULL OUTER JOIN AgeClass AS ac ON o.Offer_AgeClass_ID_FK = ac.AgeClass_ID
          JOIN BoatClass AS bc ON o.Offer_BoatClass_ID_FK = bc.BoatClass_ID
          FULL OUTER JOIN CompReferee AS cr ON cr.CompReferee_Comp_ID_FK = c.Comp_ID
          FULL OUTER JOIN Referee AS r ON r.Referee_ID = cr.CompReferee_Referee_ID_FK
          WHERE c.Comp_Event_ID_FK = @P1 AND o.Offer_RaceNumber LIKE '{filter}'
          ORDER BY c.Comp_DateTime ASC");
        info!("{}", sql);
        let mut query = Query::new(sql);
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let heats = utils::get_rows(stream).await;
        heats.into_iter().map(|row| row.to_entity()).collect()
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Kiosk {
    pub finished: Vec<Heat>,
    pub running: Vec<Heat>,
    pub next: Vec<Heat>,
}
impl Kiosk {
    pub async fn query_finished<'a>(regatta_id: i32, client: &mut AquariusClient<'_>) -> Vec<Heat> {
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

    pub async fn query_next<'a>(regatta_id: i32, client: &mut AquariusClient<'_>) -> Vec<Heat> {
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
