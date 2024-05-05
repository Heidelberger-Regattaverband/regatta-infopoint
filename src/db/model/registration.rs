use crate::db::{
    model::{utils, Club, Crew, Heat, Race, TryToEntity},
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use futures::future::{join_all, BoxFuture};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    /** The unique identifier of this registration. */
    pub id: i32,

    /** The race for which the registration was made. */
    race: Option<Race>,

    /** The club that made the registration and has to pay an entry fee for it. */
    club: Club,

    /** The crew of the registered boat. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crew: Option<Vec<Crew>>,

    /** The start number of the boat. May be None if bib number draw has not yet taken place. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bib: Option<i16>,

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
    pub cancelled: bool,

    /** An optional heat this registration is assigned to. */
    #[serde(skip_serializing_if = "Option::is_none")]
    heat: Option<Heat>,
}

impl From<&Row> for Registration {
    fn from(value: &Row) -> Self {
        let cancel_value: u8 = value.get_column("Entry_CancelValue");

        Registration {
            id: value.get_column("Entry_ID"),
            bib: value.try_get_column("Entry_Bib"),
            comment: value.try_get_column("Entry_Comment"),
            boat_number: value.try_get_column("Entry_BoatNumber"),
            short_label: value.get_column("Label_Short"),
            cancelled: cancel_value > 0,
            group_value: value.try_get_column("Entry_GroupValue"),
            club: Club::from(value),
            crew: None,
            race: value.try_to_entity(),
            heat: value.try_to_entity(),
        }
    }
}

impl Registration {
    pub(crate) fn select_columns(alias: &str) -> String {
        format!(" {0}.Entry_ID, {0}.Entry_Bib, {0}.Entry_Comment, {0}.Entry_BoatNumber, {0}.Entry_GroupValue, {0}.Entry_CancelValue ", alias)
    }

    pub(crate) async fn query_registrations_of_club(
        regatta_id: i32,
        club_id: i32,
        pool: &TiberiusPool,
    ) -> Vec<Registration> {
        let round = 64;
        let mut query = Query::new("SELECT DISTINCT".to_string() + &Registration::select_columns("e") + ", Label_Short, "
            + &Club::select_columns("oc") + ", " + &Race::select_columns("o") +
            " 
            FROM Club AS ac
            JOIN Athlet      ON Athlet_Club_ID_FK  = ac.Club_ID
            JOIN Crew        ON Crew_Athlete_ID_FK = Athlet_ID
            JOIN Entry e     ON Crew_Entry_ID_FK   = e.Entry_ID
            JOIN Club as oc  ON e.Entry_OwnerClub_ID_FK = oc.Club_ID
            JOIN EntryLabel  ON EL_Entry_ID_FK     = e.Entry_ID
            JOIN Label       ON EL_Label_ID_FK     = Label_ID
            JOIN Offer o     ON e.Entry_Race_ID_FK = o.Offer_ID
            WHERE e.Entry_Event_ID_FK = @P1 AND ac.Club_ID = @P2 AND EL_RoundFrom <= @P3 AND @P3 <= EL_RoundTo AND Crew_RoundTo = @P3
            ORDER BY o.Offer_ID ASC",
        );
        query.bind(regatta_id);
        query.bind(club_id);
        query.bind(round);

        execute_query(pool, query, round).await
    }

    pub(crate) async fn query_registrations_for_race(race_id: i32, pool: &TiberiusPool) -> Vec<Registration> {
        let round = 64;
        let mut query = Query::new(
            "SELECT DISTINCT".to_string()
                + &Registration::select_columns("e")
                + ", Label_Short, "
                + &Club::select_columns("c")
                + " 
            FROM Entry e
            JOIN EntryLabel ON EL_Entry_ID_FK   = e.Entry_ID
            JOIN Label      ON EL_Label_ID_FK   = Label_ID
            JOIN Club c     ON c.Club_ID        = Entry_OwnerClub_ID_FK
            WHERE e.Entry_Race_ID_FK = @P1 AND EL_RoundFrom <= @P2 AND @P2 <= EL_RoundTo
            ORDER BY e.Entry_Bib ASC",
        );
        query.bind(race_id);
        query.bind(round);

        execute_query(pool, query, round).await
    }
}

async fn execute_query(pool: &TiberiusPool, query: Query<'_>, round: i16) -> Vec<Registration> {
    let mut client = pool.get().await;
    let stream = query.query(&mut client).await.unwrap();

    let mut crew_futures: Vec<BoxFuture<Vec<Crew>>> = Vec::new();
    let mut registrations: Vec<Registration> = utils::get_rows(stream)
        .await
        .into_iter()
        .map(|row| {
            let registration = Registration::from(&row);
            crew_futures.push(Box::pin(Crew::query_all(registration.id, round, pool)));
            registration
        })
        .collect();

    let crews = join_all(crew_futures).await;

    for (pos, registration) in registrations.iter_mut().enumerate() {
        let crew = crews.get(pos).unwrap();
        registration.crew = Some(crew.to_vec());
    }
    registrations
}
