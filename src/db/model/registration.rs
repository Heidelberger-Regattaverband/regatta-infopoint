use crate::db::{
    aquarius::AquariusClient,
    model::{utils, Club, Crew, Race, ToEntity},
    tiberius::{RowColumn, TryRowColumn},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    pub(crate) id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    bib: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    boat_number: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    short_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_value: Option<i16>,
    club: Club,
    cancelled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) crew: Option<Vec<Crew>>,
    pub(crate) race: Race,
    #[serde(skip_serializing_if = "Option::is_none")]
    heat_date_time: Option<DateTime<Utc>>,
}

impl ToEntity<Registration> for Row {
    fn to_entity(&self) -> Registration {
        let cancel_value: u8 = self.get_column("Entry_CancelValue");
        let cancelled = cancel_value > 0;
        let id = self.get_column("Entry_ID");

        Registration {
            id,
            bib: self.try_get_column("Entry_Bib"),
            comment: self.try_get_column("Entry_Comment"),
            boat_number: self.try_get_column("Entry_BoatNumber"),
            short_label: self.get_column("Label_Short"),
            cancelled,
            group_value: self.try_get_column("Entry_GroupValue"),
            club: self.to_entity(),
            crew: None,
            race: self.to_entity(),
            heat_date_time: self.try_get_column("Heat_DateTime"),
        }
    }
}

impl Registration {
    pub async fn query_of_club(regatta_id: i32, club_id: i32, client: &mut AquariusClient<'_>) -> Vec<Registration> {
        let mut query = Query::new(
            "SELECT DISTINCT Entry.*, Label_Short, oc.Club_ID, oc.Club_Abbr, oc.Club_UltraAbbr, oc.Club_City, Offer.*,
            (SELECT MIN(Comp_DateTime) FROM Comp WHERE Comp_Race_ID_FK = Offer_ID) as Race_DateTime,
            (SELECT TOP 1 Comp_DateTime FROM Comp JOIN CompEntries ON CE_Comp_ID_FK = Comp_ID AND CE_Entry_ID_FK = Entry_ID) as Heat_DateTime
            FROM Club AS ac
            JOIN Athlet     ON Athlet_Club_ID_FK  = ac.Club_ID
            JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
            JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
            JOIN Club as oc ON Entry_OwnerClub_ID_FK = oc.Club_ID
            JOIN Event      ON Entry_Event_ID_FK  = Event_ID
            JOIN EntryLabel ON EL_Entry_ID_FK     = Entry_ID
            JOIN Label      ON EL_Label_ID_FK     = Label_ID
            JOIN Offer      ON Entry_Race_ID_FK   = Offer_ID
            WHERE Event_ID = @P1 AND ac.Club_ID = @P2 AND EL_RoundFrom <= 64 AND 64 <= EL_RoundTo
            ORDER BY Offer_ID ASC",
        );
        query.bind(regatta_id);
        query.bind(club_id);

        let stream = query.query(client).await.unwrap();
        let rows = utils::get_rows(stream).await;

        let mut registrations: Vec<Registration> = Vec::with_capacity(rows.len());
        for row in &rows {
            let mut registration: Registration = row.to_entity();
            let crew = Crew::query_all(registration.id, client).await;
            registration.crew = Some(crew);
            registrations.push(registration);
        }
        registrations
    }

    pub async fn query_for_race<'a>(race_id: i32, client: &mut AquariusClient<'_>) -> Vec<Registration> {
        let mut query = Query::new(
            "SELECT DISTINCT Entry.*, Label_Short, Club.Club_ID, Club.Club_Abbr, Club.Club_UltraAbbr, Club.Club_City, Offer.*
            FROM Entry
            JOIN EntryLabel ON EL_Entry_ID_FK   = Entry_ID
            JOIN Label      ON EL_Label_ID_FK   = Label_ID
            JOIN Club       ON Club_ID          = Entry_OwnerClub_ID_FK
            JOIN Offer      ON Entry_Race_ID_FK = Offer_ID
            WHERE Entry_Race_ID_FK = @P1 AND EL_RoundFrom <= 64 AND 64 <= EL_RoundTo
            ORDER BY Entry_Bib ASC",
        );
        query.bind(race_id);
        let stream = query.query(client).await.unwrap();
        let rows = utils::get_rows(stream).await;

        let mut registrations: Vec<Registration> = Vec::with_capacity(rows.len());
        for row in &rows {
            let mut registration: Registration = row.to_entity();
            let crew = Crew::query_all(registration.id, client).await;
            registration.crew = Some(crew);
            registrations.push(registration);
        }
        registrations
    }
}
