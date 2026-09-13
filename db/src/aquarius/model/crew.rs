use super::Club;
use super::athlete::Athlete;
use super::athlete::ID as ATHLETE_ID;
use super::club::ID as CLUB_ID;
use super::get_rows;
use crate::error::DbError;
use crate::tiberius::RowColumn;
use crate::tiberius::TiberiusPool;
use ::serde::Serialize;
use ::std::collections::HashMap;
use ::tiberius::Query;
use ::tiberius::Row;
use ::utoipa::ToSchema;

const ID: &str = "Crew_ID";
const POS: &str = "Crew_Pos";
pub(super) const IS_COX: &str = "Crew_IsCox";
const ROUND_FROM: &str = "Crew_RoundFrom";
pub(super) const ROUND_TO: &str = "Crew_RoundTo";
const ENTRY_ID_FK: &str = "Crew_Entry_ID_FK";

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

    /// Query all crew members for multiple entries in a single database round-trip.
    /// # Arguments
    /// * `entry_ids` - The entry identifiers to fetch crews for
    /// * `round` - The round of the heat; only crew members active in this round are returned
    /// * `pool` - The database connection pool
    /// # Returns
    /// A map from entry ID to its crew members, ordered by position within each entry.
    /// Entries with no crew members are absent from the map.
    /// # Errors
    /// Returns an error if the query fails or if there are issues with the database connection.
    pub(crate) async fn query_crews_for_entries(
        entry_ids: &[i32],
        round: i16,
        pool: &TiberiusPool,
    ) -> Result<HashMap<i32, Vec<Self>>, DbError> {
        if entry_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let placeholders: String = (1..=entry_ids.len())
            .map(|i| format!("@P{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let round_p = entry_ids.len() + 1;
        let sql = format!(
            "SELECT cr.{ENTRY_ID_FK}, {0}, {1}, {2} FROM Crew cr
            JOIN Athlet  a ON cr.Crew_Athlete_ID_FK = a.{ATHLETE_ID}
            JOIN Club   cl ON a.Athlet_Club_ID_FK   = cl.{CLUB_ID}
            WHERE cr.{ENTRY_ID_FK} IN ({placeholders}) AND cr.{ROUND_FROM} <= @P{round_p} AND @P{round_p} <= cr.{ROUND_TO}
            ORDER BY cr.{ENTRY_ID_FK}, cr.{POS} ASC",
            Crew::select_columns("cr"),
            Athlete::select_columns("a"),
            Club::select_all_columns("cl")
        );
        let mut query = Query::new(sql);
        for &id in entry_ids {
            query.bind(id);
        }
        query.bind(round);
        let mut client = pool.get().await?;
        let rows = get_rows(query.query(&mut client).await?).await?;
        let mut map: HashMap<i32, Vec<Crew>> = HashMap::new();
        for row in &rows {
            let entry_id: i32 = row.get_column(ENTRY_ID_FK);
            map.entry(entry_id).or_default().push(Crew::from(row));
        }
        Ok(map)
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
