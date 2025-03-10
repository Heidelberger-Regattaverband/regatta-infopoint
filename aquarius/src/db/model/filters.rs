use crate::db::{
    model::{AgeClass, Block, BoatClass, utils},
    tiberius::{RowColumn, TiberiusPool},
};
use chrono::NaiveDate;
use futures::join;
use serde::Serialize;
use tiberius::{Query, error::Error as DbError};

/// A struct containing all available filter values for a regatta.
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

    /// All blocks
    blocks: Vec<Block>,
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
    /// Query all available filter values of a regatta.
    /// # Arguments
    /// * `regatta_id` - The regatta identifier
    ///  * `pool` - The database connection pool
    /// # Returns
    /// A struct containing all available filters
    pub async fn query(regatta_id: i32, pool: &TiberiusPool) -> Result<Self, DbError> {
        let queries = join!(
            query_distances(regatta_id, pool),
            query_dates(regatta_id, pool),
            query_age_classes(regatta_id, pool),
            query_boat_classes(regatta_id, pool),
            query_rounds(regatta_id, pool),
            query_lightweight(regatta_id, pool),
            Block::query_blocks(regatta_id, pool)
        );

        Ok(Filters {
            distances: queries.0?,
            dates: queries.1?,
            age_classes: queries.2?,
            boat_classes: queries.3?,
            rounds: queries.4?,
            lightweight: queries.5?,
            blocks: queries.6?,
        })
    }
}

async fn query_boat_classes(regatta_id: i32, pool: &TiberiusPool) -> Result<Vec<BoatClass>, DbError> {
    let mut query = Query::new(format!(
        "SELECT DISTINCT {0} FROM BoatClass b
        JOIN Offer o ON o.Offer_BoatClass_ID_FK = b.BoatClass_ID 
        WHERE o.Offer_Event_ID_FK = @P1
        ORDER BY b.BoatClass_NumRowers ASC, b.BoatClass_Coxed ASC",
        BoatClass::select_columns("b")
    ));
    query.bind(regatta_id);

    let mut client = pool.get().await;
    let rows = utils::get_rows(query.query(&mut client).await?).await?;
    Ok(rows.into_iter().map(|row| BoatClass::from(&row)).collect())
}

async fn query_age_classes(regatta_id: i32, pool: &TiberiusPool) -> Result<Vec<AgeClass>, DbError> {
    let mut query = Query::new(format!(
        "SELECT DISTINCT {0} FROM AgeClass a
        JOIN Offer o ON o.Offer_AgeClass_ID_FK = a.AgeClass_ID
        WHERE o.Offer_Event_ID_FK = @P1
        ORDER BY a.AgeClass_MinAge DESC, a.AgeClass_MaxAge DESC",
        AgeClass::select_columns("a")
    ));
    query.bind(regatta_id);

    let mut client = pool.get().await;
    let rows = utils::get_rows(query.query(&mut client).await?).await?;
    Ok(rows.into_iter().map(|row| AgeClass::from(&row)).collect())
}

async fn query_dates(regatta_id: i32, pool: &TiberiusPool) -> Result<Vec<NaiveDate>, DbError> {
    let mut query = Query::new(
        "SELECT DISTINCT CAST (c.Comp_Datetime as date) AS Comp_Date
        FROM Comp c
        WHERE c.Comp_DateTime IS NOT NULL AND c.Comp_Event_ID_FK = @P1",
    );
    query.bind(regatta_id);

    let mut client = pool.get().await;
    let rows = utils::get_rows(query.query(&mut client).await?).await?;
    Ok(rows.into_iter().map(|row| row.get_column("Comp_Date")).collect())
}

async fn query_distances(regatta_id: i32, pool: &TiberiusPool) -> Result<Vec<i16>, DbError> {
    let mut query = Query::new("SELECT DISTINCT Offer_Distance FROM Offer WHERE Offer_Event_ID_FK = @P1");
    query.bind(regatta_id);

    let mut client = pool.get().await;
    let rows = utils::get_rows(query.query(&mut client).await?).await?;
    Ok(rows.into_iter().map(|row| row.get_column("Offer_Distance")).collect())
}

async fn query_lightweight(regatta_id: i32, pool: &TiberiusPool) -> Result<Vec<bool>, DbError> {
    let mut query = Query::new("SELECT DISTINCT Offer_IsLightweight FROM Offer WHERE Offer_Event_ID_FK = @P1");
    query.bind(regatta_id);

    let mut client = pool.get().await;
    let rows = utils::get_rows(query.query(&mut client).await?).await?;
    Ok(rows
        .into_iter()
        .map(|row| row.get_column("Offer_IsLightweight"))
        .collect())
}

async fn query_rounds(regatta_id: i32, pool: &TiberiusPool) -> Result<Vec<Round>, DbError> {
    let mut query: Query<'_> = Query::new(
        "SELECT DISTINCT
        c.Comp_Round, c.Comp_RoundCode
        FROM Comp c WHERE c.Comp_Event_ID_FK = @P1
        ORDER BY c.Comp_Round ASC",
    );
    query.bind(regatta_id);

    let mut client = pool.get().await;
    let rows = utils::get_rows(query.query(&mut client).await?).await?;
    Ok(rows
        .into_iter()
        .map(|row| Round {
            id: row.get_column("Comp_Round"),
            code: row.get_column("Comp_RoundCode"),
        })
        .collect())
}
