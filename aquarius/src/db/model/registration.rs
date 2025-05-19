use crate::db::{
    model::{Club, Crew, Heat, Race, TryToEntity, utils},
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use futures::future::{BoxFuture, join_all};
use serde::Serialize;
use tiberius::{Query, Row, error::Error as DbError};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    /** The unique identifier of this registration. */
    pub id: i32,

    /** The race for which the registration was made. */
    #[serde(skip_serializing_if = "Option::is_none")]
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

    /// A short label of the registration. Could be a club name or the name of a racing community.
    short_label: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    group_value: Option<i16>,

    /** Indicates whether or not the registration has been canceled. */
    pub cancelled: bool,

    /// The heats of the registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    heats: Option<Vec<Heat>>,
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
            heats: None,
        }
    }
}

impl Registration {
    pub(crate) fn select_columns(alias: &str) -> String {
        format!(
            " {0}.Entry_ID, {0}.Entry_Bib, {0}.Entry_Comment, {0}.Entry_BoatNumber, {0}.Entry_GroupValue, {0}.Entry_CancelValue ",
            alias
        )
    }

    /// Queries all registrations for a given club and regatta.
    /// # Arguments
    /// * `regatta_id` - The unique identifier of the regatta.
    /// * `club_id` - The unique identifier of the club.
    /// * `pool` - The connection pool to the database.
    /// # Returns
    /// A vector of registrations for the given club and regatta.
    pub async fn query_registrations_of_club(
        regatta_id: i32,
        club_id: i32,
        pool: &TiberiusPool,
    ) -> Result<Vec<Self>, DbError> {
        let round = 64;
        let mut query = Query::new(format!(
            "SELECT DISTINCT {0}, {1}, {2}, l.Label_Short
            FROM Club AS ac
            JOIN Athlet      a ON ac.Club_ID  = a.Athlet_Club_ID_FK
            JOIN Crew       cr ON a.Athlet_ID = cr.Crew_Athlete_ID_FK
            JOIN Entry       e ON e.Entry_ID  = cr.Crew_Entry_ID_FK 
            JOIN Club       oc ON oc.Club_ID  = e.Entry_OwnerClub_ID_FK
            JOIN EntryLabel el ON e.Entry_ID  = el.EL_Entry_ID_FK
            JOIN Label       l ON l.Label_ID  = el.EL_Label_ID_FK
            JOIN Offer       o ON o.Offer_ID  = e.Entry_Race_ID_FK
            WHERE e.Entry_Event_ID_FK = @P1 AND ac.Club_ID = @P2
                AND el.EL_RoundFrom <= @P3 AND @P3 <= el.EL_RoundTo AND cr.Crew_RoundTo = @P3
            ORDER BY o.Offer_ID ASC",
            Registration::select_columns("e"),
            Club::select_all_columns("oc"),
            Race::select_columns("o")
        ));
        query.bind(regatta_id);
        query.bind(club_id);
        query.bind(round);

        execute_query(pool, query, round).await
    }

    pub async fn query_registrations_of_athlete(
        regatta_id: i32,
        athlete_id: i32,
        pool: &TiberiusPool,
    ) -> Result<Vec<Self>, DbError> {
        let round = 64;
        let mut query = Query::new(format!(
            "SELECT DISTINCT {0}, {1}, {2}, l.Label_Short
            FROM Athlet      a
            JOIN Club       cl ON a.Athlet_Club_ID_FK  = cl.Club_ID
            JOIN Crew       cr ON a.Athlet_ID = cr.Crew_Athlete_ID_FK
            JOIN Entry       e ON e.Entry_ID  = cr.Crew_Entry_ID_FK 
            JOIN EntryLabel el ON e.Entry_ID  = el.EL_Entry_ID_FK
            JOIN Label       l ON l.Label_ID  = el.EL_Label_ID_FK
            JOIN Offer       o ON o.Offer_ID  = e.Entry_Race_ID_FK
            WHERE e.Entry_Event_ID_FK = @P1 AND a.Athlet_ID = @P2
                AND el.EL_RoundFrom <= @P3 AND @P3 <= el.EL_RoundTo AND cr.Crew_RoundTo = @P3
            ORDER BY o.Offer_ID ASC",
            Registration::select_columns("e"),
            Club::select_all_columns("cl"),
            Race::select_columns("o")
        ));
        query.bind(regatta_id);
        query.bind(athlete_id);
        query.bind(round);

        execute_query(pool, query, round).await
    }

    /// Queries all registrations for a race.
    /// # Arguments
    /// * `race_id` - The unique race identifier
    /// * `pool` - The connection pool to the database
    /// # Returns
    /// A vector of registrations for the given race
    pub async fn query_registrations_for_race(race_id: i32, pool: &TiberiusPool) -> Result<Vec<Self>, DbError> {
        let round = 64;
        let mut query = Query::new(format!(
            "SELECT DISTINCT {0}, {1}, l.Label_Short
            FROM Entry       e
            JOIN EntryLabel el ON el.EL_Entry_ID_FK = e.Entry_ID
            JOIN Label       l ON el.EL_Label_ID_FK = l.Label_ID
            JOIN Club        c ON c.Club_ID         = e.Entry_OwnerClub_ID_FK
            WHERE e.Entry_Race_ID_FK = @P1 AND el.EL_RoundFrom <= @P2 AND @P2 <= el.EL_RoundTo
            ORDER BY e.Entry_Bib ASC",
            Registration::select_columns("e"),
            Club::select_all_columns("c")
        ));
        query.bind(race_id);
        query.bind(round);

        execute_query(pool, query, round).await
    }
}

async fn execute_query(pool: &TiberiusPool, query: Query<'_>, round: i16) -> Result<Vec<Registration>, DbError> {
    let mut client = pool.get().await;
    let stream = query.query(&mut client).await?;

    let mut crew_futures: Vec<BoxFuture<Result<Vec<Crew>, DbError>>> = Vec::new();
    let mut heats_futures: Vec<BoxFuture<Result<Vec<Heat>, DbError>>> = Vec::new();
    let mut registrations: Vec<Registration> = utils::get_rows(stream)
        .await?
        .into_iter()
        .map(|row| {
            let registration = Registration::from(&row);
            crew_futures.push(Box::pin(Crew::query_crew_of_registration(registration.id, round, pool)));
            heats_futures.push(Box::pin(Heat::query_heats_of_registration(registration.id, pool)));
            registration
        })
        .collect();

    let crews = join_all(crew_futures).await;
    let heats = join_all(heats_futures).await;

    for (pos, registration) in registrations.iter_mut().enumerate() {
        let crew = crews.get(pos).unwrap().as_deref().unwrap();
        if !crew.is_empty() {
            registration.crew = Some(crew.to_vec());
        }
        let heats = heats.get(pos).unwrap().as_deref().unwrap();
        if !heats.is_empty() {
            registration.heats = Some(heats.to_vec());
        }
    }
    Ok(registrations)
}
