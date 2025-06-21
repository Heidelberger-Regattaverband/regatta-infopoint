use crate::{
    db::model::{Athlete, TryToEntity, utils},
    tiberius::{RowColumn, TiberiusPool},
};
use futures::join;
use serde::Serialize;
use tiberius::{Query, Row, error::Error as DbError};

#[derive(Debug, Serialize, Clone)]
struct RacesStatistics {
    all: i32,
    cancelled: i32,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct HeatsStatistics {
    all: i32,
    cancelled: i32,
    scheduled: i32,
    seeded: i32,
    started: i32,
    finished: i32,
    official: i32,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct EntriesStatistics {
    all: i32,
    cancelled: i32,
    registering_clubs: i32,
    athletes: i32,
    athletes_male: i32,
    athletes_female: i32,
    clubs: i32,
    seats: i32,
    seats_cox: i32,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    races: RacesStatistics,
    heats: HeatsStatistics,
    entries: EntriesStatistics,
    athletes: Option<Athletes>,
}

impl From<&Row> for Statistics {
    fn from(value: &Row) -> Self {
        let races = RacesStatistics {
            all: value.get_column("races_all"),
            cancelled: value.get_column("races_cancelled"),
        };
        let heats = HeatsStatistics {
            all: value.get_column("heats_all"),
            cancelled: value.get_column("heats_cancelled"),
            scheduled: value.get_column("heats_scheduled"),
            seeded: value.get_column("heats_seeded"),
            started: value.get_column("heats_started"),
            finished: value.get_column("heats_finished"),
            official: value.get_column("heats_official"),
        };
        let athletes_female = value.get_column("entries_athletes_female");
        let athletes_male = value.get_column("entries_athletes_male");
        let entries = EntriesStatistics {
            all: value.get_column("entries_all"),
            cancelled: value.get_column("entries_cancelled"),
            registering_clubs: value.get_column("entries_owner_clubs"),
            athletes_female,
            athletes_male,
            athletes: athletes_female + athletes_male,
            clubs: value.get_column("entries_clubs"),
            seats: value.get_column("entries_seats"),
            seats_cox: value.get_column("entries_seats_cox"),
        };
        Statistics {
            races,
            heats,
            entries,
            athletes: None,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Athletes {
    #[serde(skip_serializing_if = "Option::is_none")]
    oldest_woman: Option<Athlete>,

    #[serde(skip_serializing_if = "Option::is_none")]
    oldest_man: Option<Athlete>,
}

impl Statistics {
    pub async fn query(regatta_id: i32, pool: &TiberiusPool) -> Result<Self, DbError> {
        let mut query = Query::new(
        "SELECT
          (SELECT COUNT(*) FROM Offer WHERE Offer_Event_ID_FK = @P1) AS races_all,
          (SELECT COUNT(*) FROM Offer WHERE Offer_Event_ID_FK = @P1 AND Offer_Cancelled > 0) AS races_cancelled,
          (SELECT COUNT(*) FROM Comp  WHERE Comp_Event_ID_FK  = @P1) AS heats_all,
          (SELECT COUNT(*) FROM Comp  WHERE Comp_Event_ID_FK  = @P1 AND Comp_Cancelled > 0 ) AS heats_cancelled,
          (SELECT COUNT(*) FROM Comp  WHERE Comp_Event_ID_FK  = @P1 AND Comp_State = 4 AND Comp_Cancelled = 0 ) AS heats_official,
          (SELECT COUNT(*) FROM Comp  WHERE Comp_Event_ID_FK  = @P1 AND Comp_State = 5 OR  Comp_State = 6 ) AS heats_finished,
          (SELECT COUNT(*) FROM Comp  WHERE Comp_Event_ID_FK  = @P1 AND Comp_State = 2 AND Comp_Cancelled = 0 ) AS heats_started,
          (SELECT COUNT(*) FROM Comp  WHERE Comp_Event_ID_FK  = @P1 AND Comp_State = 1 AND Comp_Cancelled = 0 ) AS heats_seeded,
          (SELECT COUNT(*) FROM Comp  WHERE Comp_Event_ID_FK  = @P1 AND Comp_State = 0 AND Comp_Cancelled = 0 ) AS heats_scheduled,
          (SELECT COUNT(*) FROM Entry WHERE Entry_Event_ID_FK = @P1) AS entries_all,
          (SELECT COUNT(*) FROM Entry WHERE Entry_Event_ID_FK = @P1 AND Entry_CancelValue > 0) AS entries_cancelled,
          (SELECT COUNT(*) FROM (
            SELECT DISTINCT Club_ID
            FROM  Club
            JOIN  Entry ON Entry_OwnerClub_ID_FK = Club_ID
            WHERE Entry_Event_ID_FK = @P1 AND Entry_CancelValue = 0) AS count) AS entries_owner_clubs,
          (SELECT COUNT(*) FROM (
            SELECT DISTINCT Crew_Athlete_ID_FK
            FROM  Entry
            JOIN  Crew   ON Crew_Entry_ID_FK = Entry_ID
            JOIN  Athlet ON Athlet_ID        = Crew_Athlete_ID_FK
            WHERE Entry_Event_ID_FK = @P1 AND Athlet_Gender = 'M' AND Entry_CancelValue = 0) AS count) AS entries_athletes_male,
          (SELECT COUNT(*) FROM (
            SELECT DISTINCT Crew_Athlete_ID_FK
            FROM  Entry
            JOIN  Crew   ON Crew_Entry_ID_FK = Entry_ID
            JOIN  Athlet ON Athlet_ID        = Crew_Athlete_ID_FK
            WHERE Entry_Event_ID_FK = @P1 AND Athlet_Gender = 'W' AND Entry_CancelValue = 0) AS count) AS entries_athletes_female,
          (SELECT COUNT(*) FROM (
            SELECT DISTINCT Crew_Club_ID_FK
            FROM  Entry
            JOIN  Crew ON Crew_Entry_ID_FK = Entry_ID
            WHERE Entry_Event_ID_FK = @P1 AND Entry_CancelValue = 0) AS count) AS entries_clubs,
          (SELECT COALESCE(SUM(BoatClass_NumRowers), 0) FROM (
            SELECT BoatClass_NumRowers
            FROM  Entry
            JOIN  Offer     ON Offer_ID = Entry_Race_ID_FK
            JOIN  BoatClass ON BoatClass_ID = Offer_BoatClass_ID_FK
            WHERE Entry_Event_ID_FK = @P1 AND Entry_CancelValue = 0) as seats) AS entries_seats,
          (SELECT COALESCE(SUM(BoatClass_Coxed), 0) FROM (
            SELECT BoatClass_Coxed 
            FROM  Entry
            JOIN  Offer     ON Offer_ID = Entry_Race_ID_FK
            JOIN  BoatClass ON BoatClass_ID = Offer_BoatClass_ID_FK
            WHERE Entry_Event_ID_FK = @P1 AND Entry_CancelValue = 0) as seats) AS entries_seats_cox
          ",
        );
        query.bind(regatta_id);

        let mut client = pool.get().await;

        let result = join!(
            query.query(&mut client),
            Statistics::query_oldest(regatta_id, "W", pool),
            Statistics::query_oldest(regatta_id, "M", pool)
        );

        let mut stats = Statistics::from(&utils::get_row(result.0?).await?);
        stats.athletes = Some(Athletes {
            oldest_woman: result.1?,
            oldest_man: result.2?,
        });

        Ok(stats)
    }

    async fn query_oldest(regatta_id: i32, gender: &str, pool: &TiberiusPool) -> Result<Option<Athlete>, DbError> {
        let mut query = Query::new(
            "SELECT DISTINCT TOP 1 Athlet.*, Club.*
            FROM  Entry
            JOIN  Crew   ON Crew_Entry_ID_FK   = Entry_ID
            JOIN  Athlet ON Crew_Athlete_ID_FK = Athlet_ID
            JOIN  Club   ON Athlet_Club_ID_FK  = Club_ID
            WHERE Entry_Event_ID_FK = @P1 AND Entry_CancelValue = 0 AND Athlet_Gender = @P2 AND Crew_IsCox = 0
            ORDER BY Athlet_DOB",
        );
        query.bind(regatta_id);
        query.bind(gender);

        let mut client = pool.get().await;
        if let Some(row) = utils::try_get_row(query.query(&mut client).await?).await? {
            Ok(row.try_to_entity())
        } else {
            Ok(None)
        }
    }
}
