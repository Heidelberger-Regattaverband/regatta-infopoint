use crate::tiberius::TiberiusPool;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Serialize;
use tiberius::{Query, error::Error as DbError};
use utoipa::ToSchema;

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
        let mut query = Query::new(
            "SELECT c.Comp_DateTime FROM Comp c
            WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_DateTime IS NOT NULL
            ORDER BY c.Comp_DateTime ASC",
        );
        query.bind(regatta_id);

        let mut client = pool.get().await;
        let stream = query.query(&mut client).await?;
        let rows = stream.into_first_result().await?;

        let mut blocks = Vec::new();
        if !rows.is_empty() {
            let mut start: NaiveDateTime = rows[0].get(0).unwrap();
            let mut end: NaiveDateTime = rows[0].get(0).unwrap();
            let mut heats: i32 = 0;

            if rows.len() >= 2 {
                for i in 0..rows.len() - 1 {
                    let current: NaiveDateTime = rows[i].get(0).unwrap();
                    let next: NaiveDateTime = rows[i + 1].get(0).unwrap();
                    heats += 1;

                    if next.signed_duration_since(current).num_minutes() > 15 {
                        blocks.push(Block {
                            begin: start.and_utc(),
                            end: end.and_utc(),
                            heats,
                        });
                        start = next;
                        heats = 0;
                    }
                    end = next;
                }
                heats += 1;
                blocks.push(Block {
                    begin: start.and_utc(),
                    end: end.and_utc(),
                    heats,
                });
            }
        }
        Ok(blocks)
    }
}
