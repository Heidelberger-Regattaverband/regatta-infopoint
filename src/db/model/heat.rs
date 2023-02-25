use super::{Column, Race, Referee, RowColumn, ToEntity, TryToEntity};
use log::info;
use serde::Serialize;
use tiberius::{time::chrono::NaiveDateTime, Query, Row};

#[derive(Debug, Serialize, Clone)]
pub struct Heat {
    pub id: i32,
    number: i16,
    #[serde(rename = "roundCode")]
    round_code: String,
    label: String,
    #[serde(rename = "groupValue")]
    group_value: i16,
    state: u8,
    cancelled: bool,
    date: String,
    time: String,
    race: Race,
    #[serde(skip_serializing_if = "Option::is_none")]
    referee: Option<Referee>,
}

impl ToEntity<Heat> for Row {
    fn to_entity(&self) -> Heat {
        let date_time: NaiveDateTime = self.get_column("Comp_DateTime");

        Heat {
            id: self.get_column("Comp_ID"),
            race: self.to_entity(),
            number: self.get_column("Comp_Number"),
            round_code: Column::get(self, "Comp_RoundCode"),
            label: Column::get(self, "Comp_Label"),
            group_value: self.get_column("Comp_GroupValue"),
            state: self.get_column("Comp_State"),
            cancelled: self.get_column("Comp_Cancelled"),
            date: date_time.date().to_string(),
            time: date_time.time().to_string(),
            referee: self.try_to_entity(),
        }
    }
}

impl Heat {
    pub fn from_rows(rows: &Vec<Row>) -> Vec<Heat> {
        let mut heats: Vec<Heat> = Vec::with_capacity(rows.len());
        for row in rows {
            let heat = row.to_entity();
            heats.push(heat);
        }
        heats
    }

    pub(crate) fn query_all<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT DISTINCT c.*, ac.*, bc.*, r.*, hrv_o.*,
            o.Offer_RaceNumber, o.Offer_ID, o.Offer_ShortLabel, o.Offer_LongLabel, o.Offer_Comment, o.Offer_Distance, o.Offer_IsLightweight, o.Offer_Cancelled
            FROM Comp AS c
            FULL OUTER JOIN Offer AS o ON o.Offer_ID = c.Comp_Race_ID_FK
            FULL OUTER JOIN HRV_Offer AS hrv_o ON o.Offer_ID = hrv_o.id
            FULL OUTER JOIN AgeClass AS ac ON o.Offer_AgeClass_ID_FK = ac.AgeClass_ID
            JOIN BoatClass AS bc ON o.Offer_BoatClass_ID_FK = bc.BoatClass_ID
            FULL OUTER JOIN CompReferee AS cr ON cr.CompReferee_Comp_ID_FK = c.Comp_ID
            FULL OUTER JOIN Referee AS r ON r.Referee_ID = cr.CompReferee_Referee_ID_FK
            WHERE c.Comp_Event_ID_FK = @P1 ORDER BY c.Comp_DateTime ASC");
        query.bind(regatta_id);
        query
    }

    pub(crate) fn search<'a>(regatta_id: i32, filter: String) -> Query<'a> {
        let sql = format!("SELECT DISTINCT c.*, ac.*, bc.*, r.*, hrv_o.*,
          o.Offer_RaceNumber, o.Offer_ID, o.Offer_ShortLabel, o.Offer_LongLabel, o.Offer_Comment, o.Offer_Distance, o.Offer_IsLightweight, o.Offer_Cancelled
          FROM Comp AS c
          FULL OUTER JOIN Offer AS o ON o.Offer_ID = c.Comp_Race_ID_FK
          FULL OUTER JOIN HRV_Offer AS hrv_o ON o.Offer_ID = hrv_o.id
          FULL OUTER JOIN AgeClass AS ac ON o.Offer_AgeClass_ID_FK = ac.AgeClass_ID
          JOIN BoatClass AS bc ON o.Offer_BoatClass_ID_FK = bc.BoatClass_ID
          FULL OUTER JOIN CompReferee AS cr ON cr.CompReferee_Comp_ID_FK = c.Comp_ID
          FULL OUTER JOIN Referee AS r ON r.Referee_ID = cr.CompReferee_Referee_ID_FK
          WHERE c.Comp_Event_ID_FK = @P1 AND o.Offer_RaceNumber LIKE '{filter}'
          ORDER BY c.Comp_DateTime ASC");
        info!("{}", sql);
        let mut query = Query::new(sql);
        query.bind(regatta_id);
        query
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Kiosk {
    pub finished: Vec<Heat>,
    pub running: Vec<Heat>,
    pub next: Vec<Heat>,
}
impl Kiosk {
    pub(crate) fn query_finished<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT DISTINCT TOP 5 c.*, ac.*, hrv_o.*,
            o.Offer_RaceNumber, o.Offer_ID, o.Offer_ShortLabel, o.Offer_LongLabel, o.Offer_Comment, o.Offer_Distance, o.Offer_IsLightweight, o.Offer_Cancelled
            FROM Comp AS c
            FULL OUTER JOIN Offer AS o ON o.Offer_ID = c.Comp_Race_ID_FK
            FULL OUTER JOIN HRV_Offer AS hrv_o ON o.Offer_ID = hrv_o.id
            FULL OUTER JOIN AgeClass AS ac ON o.Offer_AgeClass_ID_FK = ac.AgeClass_ID
            WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_State = 4 ORDER BY c.Comp_DateTime DESC");
        query.bind(regatta_id);
        query
    }

    pub(crate) fn query_next<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT DISTINCT TOP 5 c.*, ac.*, hrv_o.*,
            o.Offer_RaceNumber, o.Offer_ID, o.Offer_ShortLabel, o.Offer_LongLabel, o.Offer_Comment, o.Offer_Distance, o.Offer_IsLightweight, o.Offer_Cancelled
            FROM Comp AS c
            FULL OUTER JOIN Offer AS o ON o.Offer_ID = c.Comp_Race_ID_FK
            FULL OUTER JOIN HRV_Offer AS hrv_o ON o.Offer_ID = hrv_o.id
            FULL OUTER JOIN AgeClass AS ac ON o.Offer_AgeClass_ID_FK = ac.AgeClass_ID
            WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_State = 1 AND c.Comp_Cancelled = 0 ORDER BY c.Comp_DateTime ASC");
        query.bind(regatta_id);
        query
    }
}
