use crate::db::{
    aquarius::AquariusClient,
    model::{utils, Athlete, ToEntity, TryToEntity},
    tiberius::RowColumn,
};
use serde::Serialize;
use tiberius::{Query, Row};

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
struct RegistrationsStatistics {
    all: i32,
    cancelled: i32,
    registering_clubs: i32,
    athletes: i32,
    clubs: i32,
    seats: i32,
    seats_cox: i32,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    races: RacesStatistics,
    heats: HeatsStatistics,
    registrations: RegistrationsStatistics,
    athletes: Option<Athletes>,
}

impl ToEntity<Statistics> for Row {
    fn to_entity(&self) -> Statistics {
        let races = RacesStatistics {
            all: self.get_column("races_all"),
            cancelled: self.get_column("races_cancelled"),
        };
        let heats = HeatsStatistics {
            all: self.get_column("heats_all"),
            cancelled: self.get_column("heats_cancelled"),
            scheduled: self.get_column("heats_scheduled"),
            seeded: self.get_column("heats_seeded"),
            started: self.get_column("heats_started"),
            finished: self.get_column("heats_finished"),
            official: self.get_column("heats_official"),
        };
        let registrations = RegistrationsStatistics {
            all: self.get_column("registrations_all"),
            cancelled: self.get_column("registrations_cancelled"),
            registering_clubs: self.get_column("registrations_owner_clubs"),
            athletes: self.get_column("registrations_athletes"),
            clubs: self.get_column("registrations_clubs"),
            seats: self.get_column("registrations_seats"),
            seats_cox: self.get_column("registrations_seats_cox"),
        };
        Statistics {
            races,
            heats,
            registrations,
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
    pub async fn query(regatta_id: i32, client: &mut AquariusClient<'_>) -> Statistics {
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
          (SELECT COUNT(*) FROM Entry WHERE Entry_Event_ID_FK = @P1) AS registrations_all,
          (SELECT COUNT(*) FROM Entry WHERE Entry_Event_ID_FK = @P1 AND Entry_CancelValue > 0) AS registrations_cancelled,
          (SELECT COUNT(*) FROM (
            SELECT DISTINCT Club_ID
            FROM  Club
            JOIN  Entry ON Entry_OwnerClub_ID_FK = Club_ID
            WHERE Entry_Event_ID_FK = @P1 AND Entry_CancelValue = 0) AS count) AS registrations_owner_clubs,
          (SELECT COUNT(*) FROM (
            SELECT DISTINCT Crew_Athlete_ID_FK
            FROM  Entry
            JOIN  Crew ON Crew_Entry_ID_FK = Entry_ID
            WHERE Entry_Event_ID_FK = @P1 AND Entry_CancelValue = 0) AS count) AS registrations_athletes,
          (SELECT COUNT(*) FROM (
            SELECT DISTINCT Crew_Club_ID_FK
            FROM  Entry
            JOIN  Crew ON Crew_Entry_ID_FK = Entry_ID
            WHERE Entry_Event_ID_FK = @P1 AND Entry_CancelValue = 0) AS count) AS registrations_clubs,
          (SELECT COALESCE(SUM(BoatClass_NumRowers), 0) FROM (
            SELECT BoatClass_NumRowers
            FROM  Entry
            JOIN  Offer     ON Offer_ID = Entry_Race_ID_FK
            JOIN  BoatClass ON BoatClass_ID = Offer_BoatClass_ID_FK
            WHERE Entry_Event_ID_FK = @P1 AND Entry_CancelValue = 0) as seats) AS registrations_seats,
          (SELECT COALESCE(SUM(BoatClass_Coxed), 0) FROM (
            SELECT BoatClass_Coxed 
            FROM  Entry
            JOIN  Offer     ON Offer_ID = Entry_Race_ID_FK
            JOIN  BoatClass ON BoatClass_ID = Offer_BoatClass_ID_FK
            WHERE Entry_Event_ID_FK = @P1 AND Entry_CancelValue = 0) as seats) AS registrations_seats_cox
          ",
        );
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let mut stats: Statistics = utils::get_row(stream).await.to_entity();

        let athletes = Athletes {
            oldest_woman: Statistics::query_oldest(regatta_id, "W", client).await,
            oldest_man: Statistics::query_oldest(regatta_id, "M", client).await,
        };
        stats.athletes = Some(athletes);

        stats
    }

    async fn query_oldest<'a>(regatta_id: i32, gender: &str, client: &mut AquariusClient<'_>) -> Option<Athlete> {
        let mut query = Query::new(
            "SELECT DISTINCT TOP 1 Athlet.*, Club.*
            FROM  Entry
            JOIN  Crew   ON Crew_Entry_ID_FK   = Entry_ID
            JOIN  Athlet ON Crew_Athlete_ID_FK = Athlet_ID
            JOIN  Club   ON Athlet_Club_ID_FK  = Club_ID
            WHERE Entry_Event_ID_FK = @P1 AND Entry_CancelValue = 0 AND Athlet_Gender = @P2
            ORDER BY Athlet_DOB",
        );
        query.bind(regatta_id);
        query.bind(gender);
        let stream = query.query(client).await.unwrap();
        if let Some(row) = utils::try_get_row(stream).await {
            row.try_to_entity()
        } else {
            None
        }
    }
}
