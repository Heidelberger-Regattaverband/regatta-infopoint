pub mod heat;
pub mod race;
pub mod statistics;

use super::utils::Column;
use serde::Serialize;
use tiberius::{time::chrono::NaiveDateTime, Query, Row};

#[derive(Debug, Serialize, Clone)]
pub struct Regatta {
    pub id: i32,
    title: String,
    sub_title: String,
    venue: String,
    #[serde(rename = "startDate")]
    start_date: String,
    #[serde(rename = "endDate")]
    end_date: String,
}
impl Regatta {
    pub fn from(row: &Row) -> Self {
        let start_date: NaiveDateTime = Column::get(row, "Event_StartDate");
        let end_date: NaiveDateTime = Column::get(row, "Event_EndDate");

        Regatta {
            id: Column::get(row, "Event_ID"),
            title: Column::get(row, "Event_Title"),
            sub_title: Column::get(row, "Event_SubTitle"),
            venue: Column::get(row, "Event_Venue"),
            start_date: start_date.date().to_string(),
            end_date: end_date.date().to_string(),
        }
    }

    pub(super) fn query_all<'a>() -> Query<'a> {
        Query::new("SELECT * FROM Event e")
    }

    pub(super) fn query_single<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT * FROM Event e WHERE e.Event_ID = @P1");
        query.bind(regatta_id);
        query
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Athlete {
    id: i32,
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
    gender: String,
    year: String,
    club: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Crew {
    id: i32,
    pos: u8,
    cox: bool,
    athlete: Athlete,
}
impl Crew {
    pub fn from(row: &Row) -> Crew {
        let dob: NaiveDateTime = Column::get(row, "Athlet_DOB");

        Crew {
            id: Column::get(row, "Crew_ID"),
            pos: Column::get(row, "Crew_Pos"),
            cox: Column::get(row, "Crew_IsCox"),
            athlete: Athlete {
                id: Column::get(row, "Athlet_ID"),
                first_name: Column::get(row, "Athlet_FirstName"),
                last_name: Column::get(row, "Athlet_LastName"),
                gender: Column::get(row, "Athlet_Gender"),
                year: dob.date().format("%Y").to_string(),
                club: Column::get(row, "Club_UltraAbbr"),
            },
        }
    }
    pub fn query_all<'a>(registration_id: i32) -> Query<'a> {
        let mut query = Query::new(
            "SELECT DISTINCT c.Crew_ID, c.Crew_Pos, c.Crew_IsCox,
                a.Athlet_ID, a.Athlet_FirstName, a.Athlet_LastName, a.Athlet_Gender, a.Athlet_DOB,
                cl.Club_UltraAbbr
            FROM Crew c
            JOIN Athlet AS a ON c.Crew_Athlete_ID_FK = a.Athlet_ID
            JOIN Club AS cl ON a.Athlet_Club_ID_FK = cl.Club_ID
            WHERE c.Crew_Entry_ID_FK = @P1 AND c.Crew_RoundFrom <= 64 AND 64 <= c.Crew_RoundTo
            ORDER BY c.Crew_pos ASC",
        );
        query.bind(registration_id);
        query
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Registration {
    pub(crate) id: i32,
    bib: i16,
    #[serde(rename = "boatNumber")]
    boat_number: i16,
    comment: String,
    #[serde(rename = "shortLabel")]
    short_label: String,
    club: Club,
    cancelled: bool,
    pub(crate) crew: Option<Vec<Crew>>,
}
impl Registration {
    pub fn from(row: &Row) -> Registration {
        let cancel_value: u8 = Column::get(row, "Entry_CancelValue");
        let cancelled = cancel_value > 0;
        let id = Column::get(row, "Entry_ID");
        Crew::query_all(id);
        Registration {
            id,
            bib: Column::get(row, "Entry_Bib"),
            comment: Column::get(row, "Entry_Comment"),
            boat_number: Column::get(row, "Entry_BoatNumber"),
            short_label: Column::get(row, "Label_Short"),
            cancelled,
            club: Club::from(row),
            crew: Option::None,
        }
    }

    pub fn query_all<'a>(race_id: i32) -> Query<'a> {
        let mut query = Query::new(
            "SELECT DISTINCT e.*, l.Label_Short, c.Club_ID, c.Club_Abbr, c.Club_City
            FROM Entry e
            JOIN EntryLabel AS el ON el.EL_Entry_ID_FK = e.Entry_ID
            JOIN Label AS l ON el.EL_Label_ID_FK = l.Label_ID
            JOIN Club AS c ON c.Club_ID = e.Entry_OwnerClub_ID_FK
            WHERE e.Entry_Race_ID_FK = @P1 AND el.EL_RoundFrom <= 64 AND 64 <= el.EL_RoundTo
            ORDER BY e.Entry_Bib ASC",
        );
        query.bind(race_id);
        query
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Club {
    id: i32,
    #[serde(rename = "shortName")]
    short_name: String,
    city: String,
}
impl Club {
    pub fn from(row: &Row) -> Self {
        Club {
            id: Column::get(row, "Club_ID"),
            short_name: Column::get(row, "Club_Abbr"),
            city: Column::get(row, "Club_City"),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Score {
    rank: i16,
    points: f64,
    club: Club,
}
impl Score {
    pub(super) fn from(row: &Row) -> Self {
        Score {
            rank: Column::get(row, "rank"),
            points: Column::get(row, "points"),
            club: Club::from(row),
        }
    }

    pub(super) fn query_all<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new(
            "SELECT s.rank, s.points, c.Club_Name, c.Club_Abbr, c.Club_City
            FROM HRV_Score s
            JOIN Club AS c ON s.club_id = c.Club_ID
            WHERE s.event_id = @P1
            ORDER BY s.rank ASC",
        );
        query.bind(regatta_id);
        query
    }
}
