# Code Review: `db` Crate

**Date:** 2026-04-20  
**Scope:** All source files in `db/src/`

---

## Summary

The `db` crate is well-structured with consistent patterns, good use of parameterized queries, and clean separation between connection management, caching, and domain models. After thorough review, the codebase is in good shape with only minor observations.

---

## Issues

### 1. `RowColumn::get_column` panics on missing columns or NULL values

- **File:** `db/src/tiberius/row_column.rs`, lines 20–23
- **Problem:** The `get_column` implementations use `.unwrap().unwrap()`, which will panic if a column is missing or contains a SQL NULL. While this is acceptable for columns known to be NOT NULL, a schema change or unexpected NULL will cause a runtime panic with no context about which column failed.
- **Suggested fix:** Consider using `.expect("column_name")` or returning `Result` to provide better diagnostics on failure.

### 2. `RowColumn<DateTime<Utc>>` silently returns epoch on error

- **File:** `db/src/tiberius/row_column.rs`, lines 36–45
- **Problem:** The `RowColumn<DateTime<Utc>>` implementation returns `DateTime::from_timestamp(0, 0).unwrap()` (Unix epoch) when `try_get` fails. This silently masks errors — the caller receives a valid-looking timestamp instead of an error.
- **Suggested fix:** Either panic with context (consistent with other `get_column` impls) or propagate the error.

### 3. `TryRowColumn` implementations silently swallow errors

- **File:** `db/src/tiberius/row_column.rs`, lines 62–116
- **Problem:** Several `TryRowColumn` implementations (e.g., for `i32`, `i16`, `u8`, `bool`, `f64`, etc.) use `unwrap_or_default()` on the outer `Result`, which means a column type mismatch error is silently treated as `None`. Only column-not-found and NULL should return `None`.
- **Suggested fix:** Distinguish between "column not found" (return `None`) and "type conversion error" (propagate or log).

### 4. `TryRowColumn<String>` treats empty strings as `None`

- **File:** `db/src/tiberius/row_column.rs`, lines 47–59
- **Fix:** Removed the empty-string check so that empty strings are now returned as `Some("".to_string())` instead of being conflated with `None`.

### 5. ~~`Timestamp::persist` redundantly calls `.to_string()` on `format!()`~~ ✅ FIXED

- **File:** `db/src/timekeeper/timestamp.rs`, lines 105–108
- **Fix:** Removed the redundant `.to_string()` call on `format!()`.

### 6. `Statistics::query` borrows `client` mutably while concurrently querying

- **File:** `db/src/aquarius/model/statistics.rs`, lines 184–189
- **Problem:** `join!` is used with `query.query(&mut client)` alongside `Statistics::query_oldest(...)` calls that also acquire their own pool connections. The main query holds a mutable borrow on `client`. While this compiles (the other queries get separate connections), it means the main statistics query and the oldest-athlete queries cannot share a connection, using 3 connections total for one logical operation.
- **Suggested fix:** This is a minor efficiency concern, not a bug. Consider sequencing the main query before the concurrent oldest-athlete queries to release the connection earlier.

### 7. ~~`Score::calculate` uses manual rank counter instead of `enumerate`~~ ✅ FIXED

- **File:** `db/src/aquarius/model/score.rs`, lines 61–70
- **Fix:** Replaced manual `index` counter with idiomatic `.enumerate()`.

### 8. `HeatResult::points` can overflow for large boats

- **File:** `db/src/aquarius/model/heat_result.rs`, line 38
- **Problem:** `num_rowers + (5 - rank)` uses `u8` arithmetic. If `rank > 5`, the expression `5 - rank` underflows (wraps in release mode). While ranks > 5 are unlikely in rowing, this is undefined-adjacent behavior.
- **Suggested fix:** Use saturating arithmetic: `num_rowers.saturating_add(5u8.saturating_sub(rank))`.

### 9. Duplicated SQL aggregation subqueries in `Club`

- **File:** `db/src/aquarius/model/club.rs`, lines 90–183
- **Problem:** `query_clubs_participating_regatta` and `query_club_with_aggregations` contain nearly identical complex subqueries for counting participations, female athletes, and male athletes. This is code duplication that increases maintenance burden.
- **Suggested fix:** Extract the common subquery logic into a shared helper method or SQL fragment builder.

### 10. `TimeStrip::add_start` and `add_finish` are nearly identical

- **File:** `db/src/timekeeper/timestrip.rs`, lines 41–63
- **Problem:** `add_start` and `add_finish` differ only in the `Split` variant passed. This is code duplication.
- **Suggested fix:** Extract a private `add_timestamp(split: Split, time: Option<DateTime<Utc>>)` method.

---

## Positive Observations

- **Parameterized queries throughout:** All SQL queries use `Query::new()` with `.bind()` for parameters — no string interpolation of user input.
- **Consistent patterns:** All model types follow `From<&Row>` + `HeapSize`/`CacheCost`, making the codebase predictable.
- **Good use of concurrent queries:** `join!`, `join3`, and `join_all` are used effectively to parallelize independent DB queries.
- **Well-designed cache layer:** The `Cache<K, V>` abstraction with `compute_if_missing` provides a clean cache-aside pattern with TTL and cost-based eviction.
- **Clean error type:** `DbError` using `thiserror` with `#[from]` conversions is idiomatic.
- **Column name constants:** Model files define column names as `const` strings, reducing typo risk.
- **Existing test coverage:** `flags_scraper` has a unit test validating the HTML parsing logic.
- **`OnceLock` for lazy globals:** `ClubFlag` and `TiberiusPool` use `OnceLock` for safe lazy initialization.