use super::get_rows;
use super::heat::DATE_TIME as HEAT_DATE_TIME;
use crate::error::DbError;
use crate::tiberius::TiberiusPool;
use crate::tiberius::TryRowColumn;
use ::chrono::DateTime;
use ::chrono::Utc;
use ::serde::Serialize;
use ::tiberius::Query;
use ::tiberius::Row;
use ::utoipa::ToSchema;

/// A block of heats.
#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct Block {
    /// Begin of the heat block
    begin: DateTime<Utc>,

    /// End of the heat block
    end: DateTime<Utc>,

    /// Number of heats in the block
    heats: i32,
}

impl Block {
    /// Query all heat blocks of the regatta. The blocks are ordered by their begin date and time.
    /// # Arguments
    /// * `regatta_id` - The unique identifier of the regatta.
    /// * `pool` - The database connection pool.
    /// # Returns
    /// A vector of `Block` structs representing the blocks
    /// # Errors
    /// Returns an error if the query fails or if there are issues with the database connection.
    pub async fn query_blocks(regatta_id: i32, pool: &TiberiusPool) -> Result<Vec<Self>, DbError> {
        let mut query = Query::new(format!(
            "SELECT c.{HEAT_DATE_TIME} FROM Comp c
            WHERE c.Comp_Event_ID_FK = @P1 AND c.{HEAT_DATE_TIME} IS NOT NULL
            ORDER BY c.{HEAT_DATE_TIME} ASC",
        ));
        query.bind(regatta_id);

        let mut client = pool.get().await?;
        let stream = query.query(&mut client).await?;
        let rows = get_rows(stream).await?;

        let mut blocks = Vec::new();
        if !rows.is_empty()
            && let Some(mut begin) = <Row as TryRowColumn<DateTime<Utc>>>::try_get_column(&rows[0], HEAT_DATE_TIME)
        {
            let mut end = begin;
            let mut heats: i32 = 0;

            if rows.len() >= 2 {
                for i in 0..rows.len() - 1 {
                    if let Some(current) =
                        <Row as TryRowColumn<DateTime<Utc>>>::try_get_column(&rows[i], HEAT_DATE_TIME)
                        && let Some(next) =
                            <Row as TryRowColumn<DateTime<Utc>>>::try_get_column(&rows[i + 1], HEAT_DATE_TIME)
                    {
                        heats += 1;

                        if next.signed_duration_since(current).num_minutes() > 15 {
                            blocks.push(Block { begin, end, heats });
                            begin = next;
                            heats = 0;
                        }
                        end = next;
                    }
                }
                heats += 1;
                blocks.push(Block { begin, end, heats });
            }
        }
        Ok(blocks)
    }
}
