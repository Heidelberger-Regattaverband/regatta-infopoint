use super::ToEntity;
use crate::db::tiberius::RowColumn;
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
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    races: RacesStatistics,
    heats: HeatsStatistics,
    registrations: RegistrationsStatistics,
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
        };
        Statistics {
            races,
            heats,
            registrations,
        }
    }
}

impl Statistics {
    pub fn query<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new(
        "SELECT
        (SELECT COUNT(*) FROM Offer WHERE Offer_Event_ID_FK = @P1) AS races_all,
        (SELECT COUNT(*) FROM Offer WHERE Offer_Event_ID_FK = @P1 AND Offer_Cancelled > 0) AS races_cancelled,
        (SELECT COUNT(*) FROM Comp  WHERE Comp_Event_ID_FK  = @P1 ) AS heats_all,
        (SELECT COUNT(*) FROM Comp  WHERE Comp_Event_ID_FK  = @P1 AND Comp_Cancelled > 0 ) AS heats_cancelled,
        (SELECT COUNT(*) FROM Comp  WHERE Comp_Event_ID_FK  = @P1 AND Comp_State = 4 AND Comp_Cancelled = 0 ) AS heats_official,
        (SELECT COUNT(*) FROM Comp  WHERE Comp_Event_ID_FK  = @P1 AND Comp_State = 5 OR  Comp_State = 6 ) AS heats_finished,
        (SELECT COUNT(*) FROM Comp  WHERE Comp_Event_ID_FK  = @P1 AND Comp_State = 2 AND Comp_Cancelled = 0 ) AS heats_started,
        (SELECT COUNT(*) FROM Comp  WHERE Comp_Event_ID_FK  = @P1 AND Comp_State = 1 AND Comp_Cancelled = 0 ) AS heats_seeded,
        (SELECT COUNT(*) FROM Comp  WHERE Comp_Event_ID_FK  = @P1 AND Comp_State = 0 AND Comp_Cancelled = 0 ) AS heats_scheduled,
        (SELECT COUNT(*) FROM Entry WHERE Entry_Event_ID_FK = @P1) AS registrations_all,
        (SELECT COUNT(*) FROM Entry WHERE Entry_Event_ID_FK = @P1 AND Entry_CancelValue > 0) AS registrations_cancelled,
        (SELECT COUNT(*) FROM (SELECT DISTINCT Club_ID FROM Club  JOIN Entry ON Entry_OwnerClub_ID_FK = Club_ID WHERE Entry_Event_ID_FK = @P1) AS count) AS registrations_owner_clubs,
        (SELECT COUNT(*) FROM (SELECT DISTINCT Crew_Athlete_ID_FK FROM Entry JOIN Crew ON Crew_Entry_ID_FK = Entry_ID WHERE Entry_Event_ID_FK = @P1) AS count) AS registrations_athletes,
        (SELECT COUNT(*) FROM (SELECT DISTINCT Crew_Club_ID_FK    FROM Entry JOIN Crew ON Crew_Entry_ID_FK = Entry_ID WHERE Entry_Event_ID_FK = @P1) AS count) AS registrations_clubs
        ",
    );
        query.bind(regatta_id);
        query
    }
}
