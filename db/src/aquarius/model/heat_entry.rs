use super::ROUND_FINAL;
use super::TryToEntity;
use super::boat_class::BC_ID;
use super::boat_class::BC_NUM_ROWERS;
use super::club::Club;
use super::club::ID as CLUB_ID;
use super::crew::Crew;
use super::entry::Entry;
use super::entry::ID as ENTRY_ID;
use super::get_rows;
use super::heat::HEAT_ID;
use super::heat::HEAT_ROUND;
use super::heat::Heat;
use super::heat_result::HeatResult;
use super::race::ID as RACE_ID;
use super::race::Race;
use crate::error::DbError;
use crate::tiberius::RowColumn;
use crate::tiberius::TiberiusPool;
use ::serde::Serialize;
use ::std::cmp::Ordering;
use ::std::time::Duration;
use ::tiberius::Query;
use ::tiberius::Row;
use ::utoipa::ToSchema;

/// A entry of a boat in a heat.
#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HeatEntry {
    /// The unique id of the entry.
    pub(crate) id: i32,

    /// The lane in which the boat is supposed to start.
    lane: i16,

    /// The entry of the boat.
    pub(crate) entry: Entry,

    /// The result of the boat in the heat
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<HeatResult>,
}

impl From<&Row> for HeatEntry {
    fn from(value: &Row) -> Self {
        HeatEntry {
            id: value.get_column("CE_ID"),
            lane: value.get_column("CE_Lane"),
            entry: Entry::from(value),
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
        let sql = format!("SELECT DISTINCT ce.CE_ID, ce.CE_Lane, {0}, Label_Short, {BC_NUM_ROWERS}, {1}, {2}, {3}
            FROM CompEntries ce
            JOIN Comp                  ON           CE_Comp_ID_FK = {HEAT_ID}
            JOIN Offer o               ON             o.{RACE_ID} = Comp_Race_ID_FK
            JOIN BoatClass             ON o.Offer_BoatClass_ID_FK = {BC_ID}
            FULL OUTER JOIN Entry e    ON          CE_Entry_ID_FK = e.{ENTRY_ID}
            FULL OUTER JOIN EntryLabel ON          EL_Entry_ID_FK = e.{ENTRY_ID}
            FULL OUTER JOIN Label      ON          EL_Label_ID_FK = Label_ID
            FULL OUTER JOIN Result r   ON       r.Result_CE_ID_FK = ce.CE_ID
            JOIN Club c                ON             c.{CLUB_ID} = Entry_OwnerClub_ID_FK
            WHERE CE_Comp_ID_FK = @P1 AND ((Result_SplitNr = {ROUND_FINAL} AND Comp_State >=4) OR (Result_SplitNr = 0 AND Comp_State < 3) OR (Comp_State < 2 AND Result_SplitNr IS NULL))
            AND EL_RoundFrom <= {HEAT_ROUND} AND {HEAT_ROUND} <= EL_RoundTo
            ORDER BY CE_Lane ASC",
            Entry::select_columns("e"), Club::select_all_columns("c"), Race::select_columns("o"), HeatResult::select_columns("r"));
        let mut query = Query::new(sql);
        query.bind(heat.id);

        let mut client = pool.get().await?;
        let rows = get_rows(query.query(&mut client).await?).await?;

        // convert rows into HeatEntry
        let mut heat_entries: Vec<HeatEntry> = rows
            .into_iter()
            .map(|row| {
                let mut heat_entry = HeatEntry::from(&row);
                // if a result is available, the entry isn't cancelled yet
                if heat_entry.result.is_some() {
                    heat_entry.entry.cancelled = false;
                }
                heat_entry
            })
            .collect();

        // sort heat entries by rank
        heat_entries.sort_by(|a, b| {
            if let (Some(result_a), Some(result_b)) = (a.result.as_ref(), b.result.as_ref()) {
                if result_a.rank_sort > result_b.rank_sort {
                    Ordering::Greater
                } else if result_a.rank_sort < result_b.rank_sort {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            } else {
                Ordering::Equal
            }
        });

        let mut first_net_time: i32 = 0;

        for (pos, heat_entry) in heat_entries.iter_mut().enumerate() {
            if let Some(result) = &mut heat_entry.result {
                if pos == 0 {
                    first_net_time = result.net_time;
                } else if result.rank_sort > 1 && result.rank_sort < u8::MAX {
                    let delta = result.net_time - first_net_time;
                    if delta > 0 {
                        let duration = Duration::from_millis(delta as u64);
                        let millis = duration.subsec_millis() / 10;
                        result.delta = Some(format!("+{}.{millis:02}", duration.as_secs()));
                    }
                }
            }
        }

        // fetch all crews in a single batch query
        let entry_ids: Vec<i32> = heat_entries.iter().map(|he| he.entry.id).collect();
        let mut crews_map = Crew::query_crews_for_entries(&entry_ids, heat.round, pool).await?;

        for heat_entry in heat_entries.iter_mut() {
            if let Some(crew) = crews_map.remove(&heat_entry.entry.id)
                && !crew.is_empty()
            {
                heat_entry.entry.crew = Some(crew);
            }
        }

        Ok(heat_entries)
    }
}
