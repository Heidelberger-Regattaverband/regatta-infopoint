use crate::db::{
    aquarius::AquariusClient,
    model::{utils, Club, Crew, Heat, Race, ToEntity, TryToEntity},
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    /** The unique identifier of this registration. */
    pub id: i32,

    /** The race for which the registration was made. */
    race: Race,

    /** The club that made the registration and has to pay an entry fee for it. */
    club: Club,

    /** The crew of the registered boat. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crew: Option<Vec<Crew>>,

    /** The start number of the boat. May be None if bib number draw has not yet taken place. */
    #[serde(skip_serializing_if = "Option::is_none")]
    bib: Option<i16>,

    /** An optional boat number, if several boats from the same club are registered. */
    #[serde(skip_serializing_if = "Option::is_none")]
    boat_number: Option<i16>,

    /** An optional comment to the registration with additional information. */
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,

    short_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_value: Option<i16>,

    /** Indicates whether or not the registration has been canceled. */
    cancelled: bool,

    /** An optional heat this registration is assigned to. */
    #[serde(skip_serializing_if = "Option::is_none")]
    heat: Option<Heat>,
}

impl ToEntity<Registration> for Row {
    fn to_entity(&self) -> Registration {
        let cancel_value: u8 = self.get_column("Entry_CancelValue");

        Registration {
            id: self.get_column("Entry_ID"),
            bib: self.try_get_column("Entry_Bib"),
            comment: self.try_get_column("Entry_Comment"),
            boat_number: self.try_get_column("Entry_BoatNumber"),
            short_label: self.get_column("Label_Short"),
            cancelled: cancel_value > 0,
            group_value: self.try_get_column("Entry_GroupValue"),
            club: self.to_entity(),
            crew: None,
            race: self.to_entity(),
            heat: self.try_to_entity(),
        }
    }
}

impl Registration {
    pub async fn query_of_club(regatta_id: i32, club_id: i32, client: &mut AquariusClient<'_>) -> Vec<Registration> {
        let mut query = Query::new(
          "SELECT DISTINCT Entry.*, Label_Short, oc.Club_ID, oc.Club_Abbr, oc.Club_UltraAbbr, oc.Club_City, Offer.*, Comp.*,
          (SELECT MIN(Comp_DateTime) FROM Comp WHERE Comp_Race_ID_FK = Offer_ID) as Race_DateTime
          FROM Club AS ac
          JOIN Athlet      ON Athlet_Club_ID_FK  = ac.Club_ID
          JOIN Crew        ON Crew_Athlete_ID_FK = Athlet_ID
          JOIN Entry       ON Crew_Entry_ID_FK   = Entry_ID
          JOIN Club as oc  ON Entry_OwnerClub_ID_FK = oc.Club_ID
          JOIN EntryLabel  ON EL_Entry_ID_FK     = Entry_ID
          JOIN Label       ON EL_Label_ID_FK     = Label_ID
          JOIN Offer       ON Entry_Race_ID_FK = Offer_ID
          JOIN CompEntries ON CE_Entry_ID_FK = Entry_ID
          JOIN Comp        ON CE_Comp_ID_FK = Comp_ID AND CE_Entry_ID_FK = Entry_ID
          WHERE Entry_Event_ID_FK = @P1 AND ac.Club_ID = @P2 AND EL_RoundFrom <= 64 AND 64 <= EL_RoundTo AND Crew_RoundTo = 64
          ORDER BY Offer_ID ASC, Comp_DateTime ASC",
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
