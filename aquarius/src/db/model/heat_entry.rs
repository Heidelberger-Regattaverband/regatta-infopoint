use crate::db::{
    model::{Club, Crew, Entry, Heat, HeatResult, Race, TryToEntity, utils},
    tiberius::{RowColumn, TiberiusPool},
};
use futures::future::{BoxFuture, join_all};
use serde::Serialize;
use std::{cmp::Ordering, time::Duration};
use tiberius::{Query, Row, error::Error as DbError};

/// A entry of a boat in a heat.
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeatEntry {
    /// The unique id of the entry.
    pub(crate) id: i32,

    /// The lane in which the boat is supposed to start.
    lane: i16,

    /// The entry of the boat.
    pub(crate) registration: Entry,

    /// The result of the boat in the heat
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<HeatResult>,
}

impl From<&Row> for HeatEntry {
    fn from(value: &Row) -> Self {
        HeatEntry {
            id: value.get_column("CE_ID"),
            lane: value.get_column("CE_Lane"),
            registration: Entry::from(value),
            result: value.try_to_entity(),
        }
    }
}

impl HeatEntry {
    /// Query all entries of a heat.
    /// # Arguments
    /// * `heat` - The heat to query the entries for
    /// * `pool` - The database connection pool
    /// # Returns
    /// A list of entries of the heat
    pub(crate) async fn query_entries_of_heat(heat: &Heat, pool: &TiberiusPool) -> Result<Vec<Self>, DbError> {
        let sql = format!("SELECT DISTINCT ce.CE_ID, ce.CE_Lane, {0}, Label_Short, BoatClass_NumRowers, {1}, {2}, {3}
            FROM CompEntries ce
            JOIN Comp                  ON CE_Comp_ID_FK     = Comp_ID
            JOIN Offer o               ON o.Offer_ID        = Comp_Race_ID_FK
            JOIN BoatClass             ON o.Offer_BoatClass_ID_FK = BoatClass_ID
            FULL OUTER JOIN Entry e    ON CE_Entry_ID_FK    = e.Entry_ID
            FULL OUTER JOIN EntryLabel ON EL_Entry_ID_FK    = e.Entry_ID
            FULL OUTER JOIN Label      ON EL_Label_ID_FK    = Label_ID
            FULL OUTER JOIN Result r   ON r.Result_CE_ID_FK = ce.CE_ID
            JOIN Club c                ON c.Club_ID = Entry_OwnerClub_ID_FK
            WHERE CE_Comp_ID_FK = @P1 AND ((Result_SplitNr = 64 AND Comp_State >=4) OR (Result_SplitNr = 0 AND Comp_State < 3) OR (Comp_State < 2 AND Result_SplitNr IS NULL))
            AND EL_RoundFrom <= Comp_Round AND Comp_Round <= EL_RoundTo", 
            Entry::select_columns("e"), Club::select_all_columns("c"), Race::select_columns("o"), HeatResult::select_columns("r"));
        let mut query = Query::new(sql);
        query.bind(heat.id);

        let mut client = pool.get().await;
        let rows = utils::get_rows(query.query(&mut client).await?).await?;

        // convert rows into HeatEntry
        let mut heat_entries: Vec<HeatEntry> = rows
            .into_iter()
            .map(|row| {
                let mut heat_entry = HeatEntry::from(&row);
                // if a result is available, the entry isn't cancelled yet
                if heat_entry.result.is_some() {
                    heat_entry.registration.cancelled = false;
                }
                heat_entry
            })
            .collect();

        // sort heat entries by rank
        heat_entries.sort_by(|a, b| {
            if let (Some(result_a), Some(result_b)) = (a.result.as_ref(), b.result.as_ref()) {
                if result_a.rank_sort > result_b.rank_sort {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            } else {
                Ordering::Equal
            }
        });

        let mut first_net_time: i32 = 0;

        let mut crew_futures: Vec<BoxFuture<Result<Vec<Crew>, DbError>>> = Vec::new();
        for (pos, heat_entry) in heat_entries.iter_mut().enumerate() {
            crew_futures.push(Box::pin(Crew::query_crew_of_entry(
                heat_entry.registration.id,
                heat.round,
                pool,
            )));

            if let Some(result) = &mut heat_entry.result {
                if pos == 0 {
                    first_net_time = result.net_time;
                } else if result.rank_sort > 1 && result.rank_sort < u8::MAX {
                    let delta = result.net_time - first_net_time;
                    let duration = Duration::from_millis(delta as u64);
                    let millis = duration.subsec_millis() / 10;
                    result.delta = Some(format!("+{}.{millis:02}", duration.as_secs()));
                }
            }
        }

        // query the crews of all entries in parallel
        let crews = join_all(crew_futures).await;

        for (pos, heat_entry) in heat_entries.iter_mut().enumerate() {
            let crew = crews.get(pos).unwrap();
            heat_entry.registration.crew = Some(crew.as_deref().unwrap().to_vec());
        }

        Ok(heat_entries)
    }
}
