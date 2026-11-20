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
use super::heat::HEAT_STATE;
use super::heat::Heat;
use super::heat_result::HeatResult;
use super::race::ID as RACE_ID;
use super::race::Race;
use crate::error::DbError;
use crate::tiberius::RowColumn;
use crate::tiberius::TiberiusPool;
use ::serde::Serialize;
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
        // States 0–3 use optional start results; states >= 4 use final results.
        let sql = format!(
            "SELECT DISTINCT ce.CE_ID, ce.CE_Lane, {0},
            COALESCE(l.Label_Short, c.Club_Abbr, c.Club_Name, '') AS Label_Short, b.{BC_NUM_ROWERS}, {1}, {2}, {3}
            FROM CompEntries ce
            JOIN Comp h            ON ce.CE_Comp_ID_FK = h.{HEAT_ID}
            JOIN Offer o           ON o.{RACE_ID} = h.Comp_Race_ID_FK
            JOIN BoatClass b       ON o.Offer_BoatClass_ID_FK = b.{BC_ID}
            JOIN Entry e           ON ce.CE_Entry_ID_FK = e.{ENTRY_ID}
            JOIN Club c            ON c.{CLUB_ID} = e.Entry_OwnerClub_ID_FK
            LEFT JOIN EntryLabel el ON el.EL_Entry_ID_FK = e.{ENTRY_ID}
                AND el.EL_RoundFrom <= h.{HEAT_ROUND} AND h.{HEAT_ROUND} <= el.EL_RoundTo
            LEFT JOIN Label l      ON el.EL_Label_ID_FK = l.Label_ID
            LEFT JOIN Result r     ON r.Result_CE_ID_FK = ce.CE_ID
                AND r.Result_SplitNr = CASE WHEN h.{HEAT_STATE} >= 4 THEN {ROUND_FINAL} ELSE 0 END
            WHERE ce.CE_Comp_ID_FK = @P1
            ORDER BY ce.CE_Lane ASC",
            Entry::select_columns("e"),
            Club::select_all_columns("c"),
            Race::select_columns("o"),
            HeatResult::select_columns("r")
        );
        let mut query = Query::new(sql);
        query.bind(heat.id);

        let mut client = pool.get().await?;
        let rows = get_rows(query.query(&mut client).await?).await?;
        // Release the completed query's connection before loading crews.
        drop(client);

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

        // Keep missing/unranked results after ranked boats so the first time is
        // the fastest available result. Stable sorting preserves lane order for ties.
        heat_entries.sort_by_key(|entry| entry.result.as_ref().map_or(u8::MAX, |result| result.rank_sort));

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

#[cfg(test)]
mod tests {
    use super::*;
    use ::tiberius::ColumnData;
    use ::tiberius::ColumnType;

    fn entry_row(rank: Option<u8>) -> Row {
        Row::builder()
            .column("CE_ID", ColumnType::Int4, ColumnData::I32(Some(10)))
            .column("CE_Lane", ColumnType::Int2, ColumnData::I16(Some(2)))
            .column("Entry_ID", ColumnType::Int4, ColumnData::I32(Some(20)))
            .column("Entry_CancelValue", ColumnType::Int1, ColumnData::U8(Some(0)))
            .column(
                "Label_Short",
                ColumnType::NVarchar,
                ColumnData::String(Some("Club".into())),
            )
            .column("Club_ID", ColumnType::Int4, ColumnData::I32(Some(30)))
            .column(
                "Club_City",
                ColumnType::NVarchar,
                ColumnData::String(Some("Heidelberg".into())),
            )
            .column(BC_NUM_ROWERS, ColumnType::Int1, ColumnData::U8(Some(2)))
            .column("Result_Rank", ColumnType::Int1, ColumnData::U8(rank))
            .column(
                "Result_NetTime",
                ColumnType::Int4,
                ColumnData::I32(rank.map(|_| 60_000)),
            )
            .column(
                "Result_DisplayValue",
                ColumnType::NVarchar,
                ColumnData::String(rank.map(|_| "1:00.00".into())),
            )
            .build()
    }

    #[test]
    fn missing_result_preserves_entry_and_lane() {
        let entry = HeatEntry::from(&entry_row(None));
        assert_eq!(entry.id, 10);
        assert_eq!(entry.lane, 2);
        assert_eq!(entry.entry.id, 20);
        assert!(entry.result.is_none());
    }

    #[test]
    fn available_result_is_decoded() {
        let entry = HeatEntry::from(&entry_row(Some(1)));
        let result = entry.result.unwrap();
        assert_eq!(result.rank_sort, 1);
        assert_eq!(result.net_time, 60_000);
    }

    #[test]
    fn unranked_result_remains_unranked() {
        let entry = HeatEntry::from(&entry_row(Some(0)));
        assert_eq!(entry.result.unwrap().rank_sort, u8::MAX);
    }
}
