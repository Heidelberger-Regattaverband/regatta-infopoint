use super::{Club, Column, Crew, RowToEntity};
use serde::Serialize;
use tiberius::{Query, Row};

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

impl RowToEntity<Registration> for Row {
    fn to_entity(&self) -> Registration {
        let cancel_value: u8 = Column::get(self, "Entry_CancelValue");
        let cancelled = cancel_value > 0;
        let id = Column::get(self, "Entry_ID");

        Registration {
            id,
            bib: Column::get(self, "Entry_Bib"),
            comment: Column::get(self, "Entry_Comment"),
            boat_number: Column::get(self, "Entry_BoatNumber"),
            short_label: Column::get(self, "Label_Short"),
            cancelled,
            club: self.to_entity(),
            crew: Option::None,
        }
    }
}

impl Registration {
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
