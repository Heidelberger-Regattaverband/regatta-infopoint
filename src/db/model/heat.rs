use super::{column::Column, race::Race, registration::Registration};
use log::info;
use serde::Serialize;
use std::time::Duration;
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

impl Heat {
    pub fn from_rows(rows: &Vec<Row>) -> Vec<Heat> {
        let mut heats: Vec<Heat> = Vec::with_capacity(rows.len());
        for row in rows {
            let heat = Heat::from_row(row);
            heats.push(heat);
        }
        heats
    }

    pub fn from_row(row: &Row) -> Self {
        let date_time: NaiveDateTime = Column::get(row, "Comp_DateTime");

        Heat {
            id: Column::get(row, "Comp_ID"),
            race: Race::from_row(row),
            number: Column::get(row, "Comp_Number"),
            round_code: Column::get(row, "Comp_RoundCode"),
            label: Column::get(row, "Comp_Label"),
            group_value: Column::get(row, "Comp_GroupValue"),
            state: Column::get(row, "Comp_State"),
            cancelled: Column::get(row, "Comp_Cancelled"),
            date: date_time.date().to_string(),
            time: date_time.time().to_string(),
            referee: Referee::from(row),
        }
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
pub struct HeatRegistration {
    pub id: i32,
    lane: i16,
    result: HeatResult,
    pub registration: Registration,
}

impl HeatRegistration {
    pub fn from_row(row: &Row) -> Self {
        HeatRegistration {
            id: Column::get(row, "CE_ID"),
            lane: Column::get(row, "CE_Lane"),
            registration: Registration::from_row(row),
            result: HeatResult::from(row),
        }
    }

    // pub fn from_rows(rows: &Vec<Row>) -> Vec<HeatRegistration> {
    //     let mut heat_regs: Vec<HeatRegistration> = Vec::with_capacity(rows.len());
    //     for row in rows {
    //         heat_regs.push(HeatRegistration::from_row(row));
    //     }
    //     heat_regs
    // }

    pub(crate) fn query_all<'a>(heat_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT DISTINCT ce.*, e.Entry_Bib, e.Entry_ID, e.Entry_BoatNumber, e.Entry_Comment, e.Entry_CancelValue, l.Label_Short, r.Result_Rank, r.Result_DisplayValue, r.Result_Delta, bc.BoatClass_NumRowers, cl.Club_ID, cl.Club_Abbr, cl.Club_City
            FROM CompEntries AS ce
            JOIN Comp AS c ON ce.CE_Comp_ID_FK = c.Comp_ID
            JOIN Offer AS o ON o.Offer_ID = c.Comp_Race_ID_FK
            JOIN BoatClass AS bc ON o.Offer_BoatClass_ID_FK = bc.BoatClass_ID
            FULL OUTER JOIN Entry AS e ON ce.CE_Entry_ID_FK = e.Entry_ID
            FULL OUTER JOIN EntryLabel AS el ON el.EL_Entry_ID_FK = e.Entry_ID
            FULL OUTER JOIN Label AS l ON el.EL_Label_ID_FK = l.Label_ID
            FULL OUTER JOIN Result AS r ON r.Result_CE_ID_FK = ce.CE_ID
            JOIN Club AS cl ON cl.Club_ID = e.Entry_OwnerClub_ID_FK
            WHERE ce.CE_Comp_ID_FK = @P1 AND (r.Result_SplitNr = 64 OR r.Result_SplitNr IS NULL)
            AND el.EL_RoundFrom <= c.Comp_Round AND c.Comp_Round <= el.EL_RoundTo");
        query.bind(heat_id);
        query
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct HeatResult {
    #[serde(rename = "rankSort")]
    rank_sort: u8,
    #[serde(rename = "rankLabel")]
    rank_label: String,
    result: String,
    delta: String,
    points: u8,
}
impl HeatResult {
    fn from(row: &Row) -> Self {
        let rank: u8 = Column::get(row, "Result_Rank");
        let rank_sort: u8 = if rank == 0 { u8::MAX } else { rank };
        let delta: String = if rank > 0 {
            let delta: i32 = Column::get(row, "Result_Delta");
            let duration = Duration::from_millis(delta as u64);
            let seconds = duration.as_secs();
            let millis = duration.subsec_millis() / 10;
            format!("{seconds}.{millis}")
        } else {
            Default::default()
        };

        let rank_label: String = if rank == 0 {
            Default::default()
        } else {
            rank.to_string()
        };

        let num_rowers: u8 = Column::get(row, "BoatClass_NumRowers");
        let points: u8 = if rank > 0 { num_rowers + (5 - rank) } else { 0 };

        HeatResult {
            delta,
            rank_label,
            rank_sort,
            result: Column::get(row, "Result_DisplayValue"),
            points,
        }
    }
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct Referee {
    id: i32,
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
}
impl Referee {
    fn from(row: &Row) -> Option<Self> {
        if let Some(id) = Column::get(row, "Referee_ID") {
            let last_name: String = Column::get(row, "Referee_LastName");
            let first_name: String = Column::get(row, "Referee_FirstName");
            if last_name.is_empty() && first_name.is_empty() {
                return None;
            }
            Some(Referee {
                id,
                last_name,
                first_name,
            })
        } else {
            None
        }
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
