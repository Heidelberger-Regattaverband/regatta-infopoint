use crate::db::tiberius::TiberiusPool;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Serialize;
use tiberius::Query;

/// A block of races.
#[derive(Debug, Serialize, Clone)]
pub struct Block {
    /// Begin of the race block
    begin: DateTime<Utc>,

    /// End of the race block
    end: DateTime<Utc>,

    /// Number of heats in the block
    heats: i32,
}

impl Block {
    /// Query all race blocks of a regatta. The blocks are ordered by their begin date and time.
    /// # Arguments
    /// * `regatta_id` - The unique identifier of the regatta.
    pub async fn query_blocks(regatta_id: i32, pool: &TiberiusPool) -> Vec<Block> {
        let mut query = Query::new(
            "SELECT c.Comp_DateTime FROM Comp c
              WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_DateTime IS NOT NULL
              ORDER BY c.Comp_DateTime ASC;",
        );
        query.bind(regatta_id);

        let mut client = pool.get().await;
        let stream = query.query(&mut client).await.unwrap();
        let rows = stream.into_first_result().await.unwrap();

        let mut start: NaiveDateTime = rows[0].get(0).unwrap();
        let mut end: NaiveDateTime = rows[0].get(0).unwrap();
        let mut heats: i32 = 0;

        let mut blocks = Vec::new();
        for i in 0..rows.len() - 2 {
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
        blocks
    }
}
