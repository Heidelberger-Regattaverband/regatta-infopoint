use super::{AgeClass, BoatClass, ToEntity};
use crate::db::{
    model::utils,
    tiberius::{RowColumn, TiberiusConnectionManager, TiberiusPool},
};
use bb8::PooledConnection;
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
}

impl Filters {
    pub async fn query(regatta_id: i32, pool: &TiberiusPool) -> Self {
        // get all available distances
        let distances = query_distances(regatta_id, pool);

        // get all available dates
        let dates = query_dates(regatta_id, pool);

        // get all used age classes
        let age_classes = query_age_classes(pool, regatta_id);

        // get all used boat classes
        let boat_classes = query_boat_classes(pool, regatta_id);

        let result = join!(distances, dates, age_classes, boat_classes);

        Filters {
            distances: result.0,
            dates: result.1,
            age_classes: result.2,
            boat_classes: result.3,
        }
    }
}

async fn query_boat_classes(pool: &TiberiusPool, regatta_id: i32) -> Vec<BoatClass> {
    let mut client: PooledConnection<'_, TiberiusConnectionManager> = pool.get().await;
    let mut query = Query::new(
        "SELECT DISTINCT b.BoatClass_ID, b.BoatClass_Caption, b.BoatClass_Abbr, b.BoatClass_NumRowers, b.BoatClass_Coxed
            FROM BoatClass b
            JOIN Offer o ON o.Offer_BoatClass_ID_FK = b.BoatClass_ID 
            WHERE o.Offer_Event_ID_FK = @P1 ORDER BY b.BoatClass_NumRowers ASC, b.BoatClass_Coxed ASC",
    );
    query.bind(regatta_id);
    let stream = query.query(&mut client).await.unwrap();
    let rows = utils::get_rows(stream).await;
    rows.into_iter().map(|row| row.to_entity()).collect()
}

async fn query_age_classes(pool: &TiberiusPool, regatta_id: i32) -> Vec<AgeClass> {
    let mut client: PooledConnection<'_, TiberiusConnectionManager> = pool.get().await;
    let mut query = Query::new(
        "SELECT DISTINCT a.AgeClass_ID, a.AgeClass_Caption, a.AgeClass_Abbr, a.AgeClass_Suffix, a.AgeClass_Gender, a.AgeClass_NumSubClasses, a.AgeClass_MinAge, a.AgeClass_MaxAge
            FROM AgeClass a
            JOIN Offer o ON o.Offer_AgeClass_ID_FK = a.AgeClass_ID
            WHERE o.Offer_Event_ID_FK = @P1
            ORDER BY a.AgeClass_MinAge DESC, a.AgeClass_MaxAge DESC",
    );
    query.bind(regatta_id);
    let stream = query.query(&mut client).await.unwrap();
    let rows = utils::get_rows(stream).await;
    rows.into_iter().map(|row| row.to_entity()).collect()
}

async fn query_dates(regatta_id: i32, pool: &TiberiusPool) -> Vec<NaiveDate> {
    let mut client = pool.get().await;
    let mut query = Query::new(
        "SELECT DISTINCT CAST (c.Comp_Datetime as date) AS Comp_Date FROM Comp c WHERE c.Comp_Event_ID_FK = @P1",
    );
    query.bind(regatta_id);
    let stream = query.query(&mut client).await.unwrap();
    let rows = utils::get_rows(stream).await;
    rows.into_iter().map(|row| row.get_column("Comp_Date")).collect()
}

async fn query_distances(regatta_id: i32, pool: &TiberiusPool) -> Vec<i16> {
    let mut client = pool.get().await;
    let mut query: Query<'_> = Query::new("SELECT DISTINCT Offer_Distance FROM Offer WHERE Offer_Event_ID_FK = @P1");
    query.bind(regatta_id);
    let stream = query.query(&mut client).await.unwrap();
    let rows = utils::get_rows(stream).await;
    rows.into_iter().map(|row| row.get_column("Offer_Distance")).collect()
}
