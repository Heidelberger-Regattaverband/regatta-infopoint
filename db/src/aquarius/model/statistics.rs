use crate::aquarius::model::boat_class::COXED;
use crate::aquarius::model::boat_class::ID as BOAT_CLASS_ID;
use crate::aquarius::model::boat_class::NUM_ROWERS;
use crate::{
    aquarius::model::{Athlete, TryToEntity, utils},
    error::DbError,
    tiberius::{RowColumn, TiberiusPool},
};
use ::futures::join;
use ::serde::Serialize;
use ::tiberius::{Query, Row};

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
    medals: MedalsStatistics,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MedalsStatistics {
    rowers: i32,
    coxes: i32,
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
        let medals = MedalsStatistics {
            rowers: value.get_column("medals_rowers"),
            coxes: value.get_column("medals_coxes"),
        };
        Statistics {
            races,
            heats,
            entries,
            athletes: None,
            medals,
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
        format!("SELECT
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
          (SELECT COALESCE(SUM({NUM_ROWERS}), 0) FROM (
            SELECT {NUM_ROWERS}
            FROM  Entry
            JOIN  Offer     ON Offer_ID = Entry_Race_ID_FK
            JOIN  BoatClass ON {BOAT_CLASS_ID} = Offer_BoatClass_ID_FK
            WHERE Entry_Event_ID_FK = @P1 AND Entry_CancelValue = 0) as seats) AS entries_seats,
          (SELECT COALESCE(SUM({COXED}), 0) FROM (
            SELECT {COXED}
            FROM  Entry      e
            JOIN  Offer      o ON o.Offer_ID         = e.Entry_Race_ID_FK
            JOIN  BoatClass bc ON bc.{BOAT_CLASS_ID} = o.Offer_BoatClass_ID_FK
            WHERE e.Entry_Event_ID_FK = @P1 AND e.Entry_CancelValue = 0) as seats) AS entries_seats_cox,
          (SELECT SUM(bc.BoatClass_NumRowers) FROM (
            SELECT bc.BoatClass_NumRowers
            FROM Comp       c
            JOIN Offer      o ON c.Comp_Race_ID_FK       =  o.Offer_ID
            JOIN BoatClass bc ON o.Offer_BoatClass_ID_FK = bc.BoatClass_ID
            WHERE c.Comp_Cancelled = 0
            ) as bc) as medals_rowers,
          (SELECT SUM(bc.BoatClass_Coxed) FROM (
            SELECT bc.BoatClass_Coxed
            FROM Comp       c
            JOIN Offer      o ON c.Comp_Race_ID_FK       =  o.Offer_ID
            JOIN BoatClass bc ON o.Offer_BoatClass_ID_FK = bc.BoatClass_ID
            WHERE c.Comp_Cancelled = 0
            ) as bc) as medals_coxes
          "
        ));
        query.bind(regatta_id);

        let mut client = pool.get().await?;
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

        let mut client = pool.get().await?;
        if let Some(row) = utils::try_get_row(query.query(&mut client).await?).await? {
            Ok(row.try_to_entity())
        } else {
            Ok(None)
        }
    }
}
