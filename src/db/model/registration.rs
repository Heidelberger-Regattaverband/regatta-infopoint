use crate::db::{
    aquarius::AquariusClient,
    model::{utils, Club, Crew, ToEntity},
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    pub(crate) id: i32,
    bib: i16,
    #[serde(skip_serializing_if = "Option::is_none")]
    boat_number: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    short_label: String,
    club: Club,
    cancelled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) crew: Option<Vec<Crew>>,
}

impl ToEntity<Registration> for Row {
    fn to_entity(&self) -> Registration {
        let cancel_value: u8 = self.get_column("Entry_CancelValue");
        let cancelled = cancel_value > 0;
        let id = self.get_column("Entry_ID");

        Registration {
            id,
            bib: self.get_column("Entry_Bib"),
            comment: self.try_get_column("Entry_Comment"),
            boat_number: self.try_get_column("Entry_BoatNumber"),
            short_label: self.get_column("Label_Short"),
            cancelled,
            club: self.to_entity(),
            crew: Option::None,
        }
    }
}

impl Registration {
    pub async fn query_of_clubs(regatta_id: i32, club_id: i32, client: &mut AquariusClient<'_>) -> Vec<Registration> {
        let mut query = Query::new(
            "SELECT DISTINCT Entry_ID, Entry_CancelValue, Entry_Bib, Entry_Comment, Entry_BoatNumber, Label_Short, Club_ID, Club_Name, Club_Abbr, Club_UltraAbbr, Club_City
            FROM Club
            JOIN Athlet     ON Athlet_Club_ID_FK  = Club_ID
            JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
            JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
            JOIN Event      ON Entry_Event_ID_FK  = Event_ID
            JOIN EntryLabel ON EL_Entry_ID_FK = Entry_ID
            JOIN Label      ON EL_Label_ID_FK = Label_ID
            WHERE Event_ID = @P1 AND Club_ID = @P2 AND EL_RoundFrom <= 64 AND 64 <= EL_RoundTo
            ORDER BY Club_City ASC",
        );
        query.bind(regatta_id);
        query.bind(club_id);

        let stream = query.query(client).await.unwrap();
        let registrations = utils::get_rows(stream).await;
        registrations.into_iter().map(|row| row.to_entity()).collect()
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
