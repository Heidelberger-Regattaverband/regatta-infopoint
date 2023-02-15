pub mod crew;
pub mod heat;
pub mod race;
pub mod score;
pub mod statistics;

use super::utils::Column;
use crew::Crew;
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
    pub fn from_rows(rows: &Vec<Row>) -> Vec<Self> {
        let mut regattas: Vec<Regatta> = Vec::with_capacity(rows.len());
        for row in rows {
            let regatta = Regatta::from_row(row);
            regattas.push(regatta);
        }
        regattas
    }

    pub fn from_row(row: &Row) -> Self {
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
    pub fn from_row(row: &Row) -> Registration {
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
