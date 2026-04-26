use super::get_rows;
use crate::{error::DbError, tiberius::TiberiusPool};
use ::serde::Serialize;
use ::std::collections::HashMap;
use ::tiberius::Query;
use ::utoipa::ToSchema;

/// Maximum number of boats per heat (number of lanes).
const LANES: usize = 4;

/// A race that has club conflicts in its hypothetical heat assignments.
#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClubConflictRace {
    /// The race identifier.
    race_id: i32,

    /// The race number, e.g. "15" or "115a".
    race_number: String,

    /// Short label of the race, e.g. "JM 2x".
    race_short_label: String,

    /// Long label of the race.
    race_long_label: String,

    /// The heats that contain club conflicts.
    heats: Vec<ConflictHeat>,
}

/// A hypothetical heat that contains multiple boats from the same club.
#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConflictHeat {
    /// The heat number (1-based).
    heat_number: usize,

    /// The club conflicts in this heat.
    conflicts: Vec<ClubConflict>,
}

/// Boats from the same club that ended up in the same heat.
#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClubConflict {
    /// The club identifier.
    club_id: i32,

    /// The club name (abbreviation).
    club_name: String,

    /// The bibs (start numbers) of the club's boats in this heat.
    bibs: Vec<i16>,
}

/// An intermediate struct to hold the raw entry data from the query.
struct RaceEntryRow {
    race_id: i32,
    race_number: String,
    race_short_label: String,
    race_long_label: String,
    bib: i16,
    club_id: i32,
    club_name: String,
}

impl ClubConflictRace {
    /// Query all races of a regatta that have club conflicts in their hypothetical heat assignments.
    ///
    /// The boats in a race are assigned in the order of their bib numbers into heats of
    /// maximum [`LANES`] boats. If two or more boats from the same club end up in the same
    /// heat, the race is reported as a conflict.
    ///
    /// # Arguments
    /// * `regatta_id` - The regatta identifier
    /// * `pool` - The database connection pool
    ///
    /// # Returns
    /// A list of races with club conflicts
    pub async fn query_club_conflicts(regatta_id: i32, pool: &TiberiusPool) -> Result<Vec<Self>, DbError> {
        // Query all non-cancelled entries with bib numbers, ordered by race and bib.
        // We join Entry with its owning Club and the Race (Offer).
        let sql = "SELECT o.Offer_ID AS RaceId, o.Offer_RaceNumber AS RaceNumber, \
                    o.Offer_ShortLabel AS RaceShortLabel, o.Offer_LongLabel AS RaceLongLabel, \
                    e.Entry_Bib AS Bib, c.Club_ID AS ClubId, c.Club_Abbr AS ClubName \
             FROM Entry e \
             JOIN Offer o ON o.Offer_ID = e.Entry_Race_ID_FK \
             JOIN Club  c ON c.Club_ID  = e.Entry_OwnerClub_ID_FK \
             WHERE e.Entry_Event_ID_FK = @P1 \
               AND e.Entry_CancelValue = 0 \
               AND e.Entry_Bib IS NOT NULL \
             ORDER BY o.Offer_SortValue ASC, e.Entry_Bib ASC";

        let mut query = Query::new(sql);
        query.bind(regatta_id);

        let mut client = pool.get().await?;
        let rows = get_rows(query.query(&mut client).await?).await?;

        // Parse rows into intermediate structs
        use crate::tiberius::RowColumn;
        let entries: Vec<RaceEntryRow> = rows
            .iter()
            .map(|row| {
                let short_label: String = row.get_column("RaceShortLabel");
                let long_label: String = row.get_column("RaceLongLabel");
                RaceEntryRow {
                    race_id: row.get_column("RaceId"),
                    race_number: row.get_column("RaceNumber"),
                    race_short_label: short_label.trim().to_owned(),
                    race_long_label: long_label.trim().to_owned(),
                    bib: row.get_column("Bib"),
                    club_id: row.get_column("ClubId"),
                    club_name: row.get_column("ClubName"),
                }
            })
            .collect();

        Ok(find_club_conflicts(entries))
    }
}

/// Find races where boats from the same club end up in the same heat.
///
/// Entries are already sorted by race and bib. We group them by race,
/// then assign each entry to a heat based on its bib number:
/// heat = (bib - 1) / LANES + 1, i.e. bibs 1-4 → heat 1, bibs 5-8 → heat 2, etc.
fn find_club_conflicts(entries: Vec<RaceEntryRow>) -> Vec<ClubConflictRace> {
    // Group entries by race_id, preserving order
    let mut races_order: Vec<i32> = Vec::new();
    let mut entries_by_race: HashMap<i32, Vec<&RaceEntryRow>> = HashMap::new();

    for entry in &entries {
        entries_by_race
            .entry(entry.race_id)
            .or_insert_with(|| {
                races_order.push(entry.race_id);
                Vec::new()
            })
            .push(entry);
    }

    let mut result = Vec::new();

    for race_id in &races_order {
        if let Some(race_entries) = entries_by_race.get(race_id) {
            // Assign entries to heats based on bib number (bibs start at 1).
            // Heat number = (bib - 1) / LANES + 1, so bibs 1-4 → heat 1, bibs 5-8 → heat 2, etc.
            let mut heats_map: HashMap<usize, Vec<&RaceEntryRow>> = HashMap::new();
            for entry in race_entries {
                let heat_number = ((entry.bib as usize - 1) / LANES) + 1;
                heats_map.entry(heat_number).or_default().push(entry);
            }

            let mut conflict_heats: Vec<ConflictHeat> = heats_map
                .into_iter()
                .filter_map(|(heat_number, heat_entries)| find_conflicts_in_heat(heat_number, &heat_entries))
                .collect();

            // Sort heats by heat number for consistent output
            conflict_heats.sort_by_key(|h| h.heat_number);

            if !conflict_heats.is_empty() {
                let first = race_entries[0];
                result.push(ClubConflictRace {
                    race_id: first.race_id,
                    race_number: first.race_number.clone(),
                    race_short_label: first.race_short_label.clone(),
                    race_long_label: first.race_long_label.clone(),
                    heats: conflict_heats,
                });
            }
        }
    }

    result
}

/// Check if a hypothetical heat has club conflicts (multiple boats from the same club).
fn find_conflicts_in_heat(heat_number: usize, entries: &[&RaceEntryRow]) -> Option<ConflictHeat> {
    // Group bibs by club_id
    let mut by_club: HashMap<i32, Vec<i16>> = HashMap::new();
    let mut club_names: HashMap<i32, &str> = HashMap::new();

    for entry in entries {
        by_club.entry(entry.club_id).or_default().push(entry.bib);
        club_names.entry(entry.club_id).or_insert(&entry.club_name);
    }

    // Find clubs with more than one boat in this heat
    let conflicts: Vec<ClubConflict> = by_club
        .into_iter()
        .filter(|(_, bibs)| bibs.len() > 1)
        .map(|(club_id, bibs)| ClubConflict {
            club_id,
            club_name: club_names.get(&club_id).unwrap_or(&"").to_string(),
            bibs,
        })
        .collect();

    if conflicts.is_empty() {
        None
    } else {
        Some(ConflictHeat { heat_number, conflicts })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(race_id: i32, race_number: &str, bib: i16, club_id: i32, club_name: &str) -> RaceEntryRow {
        RaceEntryRow {
            race_id,
            race_number: race_number.to_string(),
            race_short_label: format!("Race {race_number}"),
            race_long_label: format!("Race {race_number} Long"),
            bib,
            club_id,
            club_name: club_name.to_string(),
        }
    }

    #[test]
    fn test_no_conflicts() {
        let entries = vec![
            make_entry(1, "1", 1, 10, "Club A"),
            make_entry(1, "1", 2, 20, "Club B"),
            make_entry(1, "1", 3, 30, "Club C"),
            make_entry(1, "1", 4, 40, "Club D"),
        ];
        let result = find_club_conflicts(entries);
        assert!(result.is_empty());
    }

    #[test]
    fn test_conflict_same_heat() {
        // Bibs 1-4 go into heat 1. Club A has bibs 1 and 3.
        let entries = vec![
            make_entry(1, "1", 1, 10, "Club A"),
            make_entry(1, "1", 2, 20, "Club B"),
            make_entry(1, "1", 3, 10, "Club A"),
            make_entry(1, "1", 4, 30, "Club C"),
        ];
        let result = find_club_conflicts(entries);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].race_id, 1);
        assert_eq!(result[0].heats.len(), 1);
        assert_eq!(result[0].heats[0].heat_number, 1);
        assert_eq!(result[0].heats[0].conflicts.len(), 1);
        assert_eq!(result[0].heats[0].conflicts[0].club_id, 10);
        assert_eq!(result[0].heats[0].conflicts[0].bibs, vec![1, 3]);
    }

    #[test]
    fn test_no_conflict_different_heats() {
        // Bibs 1-4 → heat 1, bibs 5-8 → heat 2
        // Club A has bib 1 (heat 1) and bib 5 (heat 2) → no conflict
        let entries = vec![
            make_entry(1, "1", 1, 10, "Club A"),
            make_entry(1, "1", 2, 20, "Club B"),
            make_entry(1, "1", 3, 30, "Club C"),
            make_entry(1, "1", 4, 40, "Club D"),
            make_entry(1, "1", 5, 10, "Club A"),
            make_entry(1, "1", 6, 50, "Club E"),
            make_entry(1, "1", 7, 60, "Club F"),
            make_entry(1, "1", 8, 70, "Club G"),
        ];
        let result = find_club_conflicts(entries);
        assert!(result.is_empty());
    }

    #[test]
    fn test_conflict_in_second_heat() {
        // Heat 1: bibs 1-4 (no conflict), Heat 2: bibs 5-8 (Club B has 6 and 7)
        let entries = vec![
            make_entry(1, "1", 1, 10, "Club A"),
            make_entry(1, "1", 2, 20, "Club B"),
            make_entry(1, "1", 3, 30, "Club C"),
            make_entry(1, "1", 4, 40, "Club D"),
            make_entry(1, "1", 5, 50, "Club E"),
            make_entry(1, "1", 6, 20, "Club B"),
            make_entry(1, "1", 7, 20, "Club B"),
            make_entry(1, "1", 8, 60, "Club F"),
        ];
        let result = find_club_conflicts(entries);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].heats.len(), 1);
        assert_eq!(result[0].heats[0].heat_number, 2);
        assert_eq!(result[0].heats[0].conflicts[0].club_id, 20);
        assert_eq!(result[0].heats[0].conflicts[0].bibs, vec![6, 7]);
    }

    #[test]
    fn test_multiple_races_only_one_has_conflict() {
        let entries = vec![
            // Race 1: no conflict
            make_entry(1, "1", 1, 10, "Club A"),
            make_entry(1, "1", 2, 20, "Club B"),
            make_entry(1, "1", 3, 30, "Club C"),
            make_entry(1, "1", 4, 40, "Club D"),
            // Race 2: conflict (Club X has bibs 1 and 2)
            make_entry(2, "2", 1, 100, "Club X"),
            make_entry(2, "2", 2, 100, "Club X"),
            make_entry(2, "2", 3, 200, "Club Y"),
        ];
        let result = find_club_conflicts(entries);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].race_id, 2);
    }

    #[test]
    fn test_multiple_conflicts_in_same_heat() {
        // Heat 1: Club A has bibs 1,2 and Club B has bibs 3,4
        let entries = vec![
            make_entry(1, "1", 1, 10, "Club A"),
            make_entry(1, "1", 2, 10, "Club A"),
            make_entry(1, "1", 3, 20, "Club B"),
            make_entry(1, "1", 4, 20, "Club B"),
        ];
        let result = find_club_conflicts(entries);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].heats[0].conflicts.len(), 2);
    }

    #[test]
    fn test_bib_based_heat_assignment_with_gaps() {
        // Bibs 1 and 3 exist (heat 1), bib 5 exists (heat 2).
        // Even though only 3 entries, bib 5 goes to heat 2 based on its value.
        // Club A has bibs 1 and 3 → both in heat 1 → conflict
        let entries = vec![
            make_entry(1, "1", 1, 10, "Club A"),
            make_entry(1, "1", 3, 10, "Club A"),
            make_entry(1, "1", 5, 20, "Club B"),
        ];
        let result = find_club_conflicts(entries);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].heats[0].heat_number, 1);
        assert_eq!(result[0].heats[0].conflicts[0].bibs, vec![1, 3]);
    }

    #[test]
    fn test_bib_based_heat_boundary() {
        // Bib 4 → heat 1, bib 5 → heat 2. Same club but different heats → no conflict
        let entries = vec![make_entry(1, "1", 4, 10, "Club A"), make_entry(1, "1", 5, 10, "Club A")];
        let result = find_club_conflicts(entries);
        assert!(result.is_empty());
    }
}
