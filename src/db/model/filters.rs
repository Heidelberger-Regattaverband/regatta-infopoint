use crate::db::{
    model::{utils, AgeClass, BoatClass},
    tiberius::{RowColumn, TiberiusPool},
};
use chrono::NaiveDate;
use futures::join;
use serde::Serialize;
use tiberius::Query;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Filters {
    /// Available distances of all races.
    distances: Vec<i16>,

    /// All dates of the races.
    dates: Vec<NaiveDate>,

    /// All age classes used in races
    age_classes: Vec<AgeClass>,

    /// All boat classes used in races
    boat_classes: Vec<BoatClass>,

    /// All rounds: 4 - Vorlauf, 16 - Hoffnungslauf, 32 - Semifinal, 64 - Final
    rounds: Vec<Round>,

    /// All used lightweight categories
    lightweight: Vec<bool>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Round {
    /// the round id: 4 - Vorlauf, 16 - Hoffnungslauf, 32 - Semifinal, 64 - Final
    id: i16,

    /// The code: V - Vorlauf, H - Hoffnungslauf
    code: String,
}

impl Filters {
    pub async fn query(regatta_id: i32, pool: &TiberiusPool) -> Self {
        // get all available distances
        let distances = query_distances(regatta_id, pool);

        // get all available dates
        let dates = query_dates(regatta_id, pool);

        // get all used age classes
        let age_classes = query_age_classes(regatta_id, pool);

        // get all used boat classes
        let boat_classes = query_boat_classes(regatta_id, pool);

        let rounds = query_rounds(regatta_id, pool);

        let lightweight = query_lightweight(regatta_id, pool);

        let result = join!(distances, dates, age_classes, boat_classes, rounds, lightweight);

        Filters {
            distances: result.0,
            dates: result.1,
            age_classes: result.2,
            boat_classes: result.3,
            rounds: result.4,
            lightweight: result.5,
        }
    }
}

async fn query_boat_classes(regatta_id: i32, pool: &TiberiusPool) -> Vec<BoatClass> {
    let mut query = Query::new(
        "SELECT DISTINCT".to_string()
            + &BoatClass::select_columns("b")
            + "FROM BoatClass b
            JOIN Offer o ON o.Offer_BoatClass_ID_FK = b.BoatClass_ID 
            WHERE o.Offer_Event_ID_FK = @P1 ORDER BY b.BoatClass_NumRowers ASC, b.BoatClass_Coxed ASC",
    );
    query.bind(regatta_id);

    let mut client = pool.get().await;
    let rows = utils::get_rows(query.query(&mut client).await.unwrap()).await;
    rows.into_iter().map(|row| BoatClass::from(&row)).collect()
}

async fn query_age_classes(regatta_id: i32, pool: &TiberiusPool) -> Vec<AgeClass> {
    let mut query = Query::new(
        "SELECT DISTINCT".to_string()
            + &AgeClass::select_columns("a")
            + "FROM AgeClass a
            JOIN Offer o ON o.Offer_AgeClass_ID_FK = a.AgeClass_ID
            WHERE o.Offer_Event_ID_FK = @P1
            ORDER BY a.AgeClass_MinAge DESC, a.AgeClass_MaxAge DESC",
    );
    query.bind(regatta_id);

    let mut client = pool.get().await;
    let rows = utils::get_rows(query.query(&mut client).await.unwrap()).await;
    rows.into_iter().map(|row| AgeClass::from(&row)).collect()
}

async fn query_dates(regatta_id: i32, pool: &TiberiusPool) -> Vec<NaiveDate> {
    let mut query = Query::new(
        "SELECT DISTINCT CAST (c.Comp_Datetime as date) AS Comp_Date
        FROM Comp c
        WHERE c.Comp_DateTime IS NOT NULL AND c.Comp_Event_ID_FK = @P1",
    );
    query.bind(regatta_id);

    let mut client = pool.get().await;
    let rows = utils::get_rows(query.query(&mut client).await.unwrap()).await;
    rows.into_iter().map(|row| row.get_column("Comp_Date")).collect()
}

async fn query_distances(regatta_id: i32, pool: &TiberiusPool) -> Vec<i16> {
    let mut query: Query<'_> = Query::new("SELECT DISTINCT Offer_Distance FROM Offer WHERE Offer_Event_ID_FK = @P1");
    query.bind(regatta_id);

    let mut client = pool.get().await;
    let rows = utils::get_rows(query.query(&mut client).await.unwrap()).await;
    rows.into_iter().map(|row| row.get_column("Offer_Distance")).collect()
}

async fn query_lightweight(regatta_id: i32, pool: &TiberiusPool) -> Vec<bool> {
    let mut query: Query<'_> =
        Query::new("SELECT DISTINCT Offer_IsLightweight FROM Offer WHERE Offer_Event_ID_FK = @P1");
    query.bind(regatta_id);

    let mut client = pool.get().await;
    let rows = utils::get_rows(query.query(&mut client).await.unwrap()).await;
    rows.into_iter()
        .map(|row| row.get_column("Offer_IsLightweight"))
        .collect()
}

async fn query_rounds(regatta_id: i32, pool: &TiberiusPool) -> Vec<Round> {
    let mut query: Query<'_> = Query::new(
        "SELECT DISTINCT
        c.Comp_Round, c.Comp_RoundCode
        FROM Comp c WHERE c.Comp_Event_ID_FK = @P1
        ORDER BY c.Comp_Round ASC",
    );
    query.bind(regatta_id);

    let mut client = pool.get().await;
    let rows = utils::get_rows(query.query(&mut client).await.unwrap()).await;
    rows.into_iter()
        .map(|row| Round {
            id: row.get_column("Comp_Round"),
            code: row.get_column("Comp_RoundCode"),
        })
        .collect()
}
