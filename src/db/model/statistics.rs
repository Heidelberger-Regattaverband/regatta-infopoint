use super::column::Column;
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
struct RacesStatistics {
    all: i32,
    cancelled: i32,
}

#[derive(Debug, Serialize, Clone)]
struct HeatsStatistics {
    all: i32,
    cancelled: i32,
    pending: i32,
    started: i32,
    finished: i32,
    official: i32,
}

#[derive(Debug, Serialize, Clone)]
struct RegistrationsStatistics {
    all: i32,
    cancelled: i32,
    #[serde(rename = "registeringClubs")]
    registering_clubs: i32,
    athletes: i32,
    clubs: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct Statistics {
    races: RacesStatistics,
    heats: HeatsStatistics,
    registrations: RegistrationsStatistics,
}

impl Statistics {
    pub(crate) fn from_row(row: &Row) -> Self {
        let races = RacesStatistics {
            all: Column::get(row, "races_all"),
            cancelled: Column::get(row, "races_cancelled"),
        };
        let heats = HeatsStatistics {
            all: Column::get(row, "heats_all"),
            cancelled: Column::get(row, "heats_cancelled"),
            finished: Column::get(row, "heats_finished"),
            official: Column::get(row, "heats_official"),
            pending: Column::get(row, "heats_pending"),
            started: Column::get(row, "heats_started"),
        };
        let registrations = RegistrationsStatistics {
            all: Column::get(row, "registrations_all"),
            cancelled: Column::get(row, "registrations_cancelled"),
            registering_clubs: Column::get(row, "registrations_owner_clubs"),
            athletes: Column::get(row, "registrations_athletes"),
            clubs: Column::get(row, "registrations_clubs"),
        };
        Statistics {
            races,
            heats,
            registrations,
        }
    }

    pub(crate) fn query<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new(
            "SELECT
            (SELECT COUNT(*) FROM Offer o WHERE o.Offer_Event_ID_FK = @P1) AS races_all,
            (SELECT COUNT(*) FROM Offer o WHERE o.Offer_Event_ID_FK = @P1 AND o.Offer_Cancelled > 0) AS races_cancelled,
            (SELECT COUNT(*) FROM Comp c WHERE c.Comp_Event_ID_FK = @P1 ) AS heats_all,
            (SELECT COUNT(*) FROM Comp c WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_Cancelled > 0 ) AS heats_cancelled,
            (SELECT COUNT(*) FROM Comp c WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_State = 4 ) AS heats_official,
            (SELECT COUNT(*) FROM Comp c WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_State = 5 OR c.Comp_State = 6 ) AS heats_finished,
            (SELECT COUNT(*) FROM Comp c WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_State = 2 ) AS heats_started,
            (SELECT COUNT(*) FROM Comp c WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_State < 2 AND c.Comp_Cancelled = 0 ) AS heats_pending,
            (SELECT COUNT(*) FROM Entry e WHERE e.Entry_Event_ID_FK = @P1) AS registrations_all,
            (SELECT COUNT(*) FROM Entry e WHERE e.Entry_Event_ID_FK = @P1 AND e.Entry_CancelValue > 0) AS registrations_cancelled,
            (SELECT COUNT(*) FROM (SELECT DISTINCT c.Club_ID FROM Club c JOIN Entry e ON e.Entry_OwnerClub_ID_FK = c.Club_ID WHERE e.Entry_Event_ID_FK = @P1) AS count) AS registrations_owner_clubs,
            (SELECT COUNT(*) FROM (SELECT DISTINCT c.Crew_Athlete_ID_FK FROM Entry e JOIN Crew c ON c.Crew_Entry_ID_FK = e.Entry_ID WHERE e.Entry_Event_ID_FK = @P1) AS count) AS registrations_athletes,
            (SELECT COUNT(*) FROM (SELECT DISTINCT c.Crew_Club_ID_FK FROM Entry e JOIN Crew c ON c.Crew_Entry_ID_FK = e.Entry_ID WHERE e.Entry_Event_ID_FK = @P1) AS count) AS registrations_clubs
          ",
        );
        query.bind(regatta_id);
        query
    }
}
