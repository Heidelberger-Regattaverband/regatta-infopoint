use crate::{
    aquarius::model::{Athlete, Club, utils},
    error::DbError,
    tiberius::{RowColumn, TiberiusPool},
};
use serde::Serialize;
use tiberius::{Query, Row};
use utoipa::ToSchema;

const ID: &str = "Crew_ID";
const POS: &str = "Crew_Pos";
const IS_COX: &str = "Crew_IsCox";
const ROUND_FROM: &str = "Crew_RoundFrom";
const ROUND_TO: &str = "Crew_RoundTo";

#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Crew {
    id: i32,

    /// Position in the boat
    pos: u8,

    /// Is cox in the boat
    cox: bool,

    /// the athlete
    athlete: Athlete,

    /// Is crew member from round.
    round_from: i16,

    /// Is crew member until round.
    round_to: i16,
}

impl Crew {
    pub(crate) fn select_columns(alias: &str) -> String {
        format!("{alias}.{ID}, {alias}.{POS}, {alias}.{IS_COX}, {alias}.{ROUND_FROM}, {alias}.{ROUND_TO}")
    }

    /// Query all crew members of a entry.
    /// # Arguments
    /// * `entry_id` - The entry identifier
    /// * `round` - The round of the heat this crew is participating in
    /// * `pool` - The database connection pool
    /// # Returns
    /// A list of crew members of the entry
    pub(crate) async fn query_crew_of_entry(
        entry_id: i32,
        round: i16,
        pool: &TiberiusPool,
    ) -> Result<Vec<Self>, DbError> {
        let sql = format!(
            "SELECT {0}, {1}, {2} FROM Crew cr
            JOIN Athlet  a ON cr.Crew_Athlete_ID_FK = a.Athlet_ID
            JOIN Club   cl ON a.Athlet_Club_ID_FK   = cl.Club_ID
            WHERE Crew_Entry_ID_FK = @P1 AND cr.Crew_RoundFrom <= @P2 AND @P2 <= cr.Crew_RoundTo
            ORDER BY cr.Crew_pos ASC",
            Crew::select_columns("cr"),
            Athlete::select_columns("a"),
            Club::select_all_columns("cl")
        );
        let mut query = Query::new(sql);
        query.bind(entry_id);
        query.bind(round);

        let mut client = pool.get().await?;
        let crew = utils::get_rows(query.query(&mut client).await?).await?;
        Ok(crew.into_iter().map(|row| Crew::from(&row)).collect())
    }
}

impl From<&Row> for Crew {
    fn from(value: &Row) -> Self {
        Crew {
            id: value.get_column(ID),
            pos: value.get_column(POS),
            cox: value.get_column(IS_COX),
            athlete: Athlete::from(value),
            round_from: value.get_column(ROUND_FROM),
            round_to: value.get_column(ROUND_TO),
        }
    }
}
