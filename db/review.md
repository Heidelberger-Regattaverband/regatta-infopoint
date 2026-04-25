# Code Review: `db` Crate

**Date:** 2026-04-24  
**Scope:** All source files in `db/src/`  
**Clippy:** ✅ Clean (no warnings)

---

## Summary

The `db` crate is well-structured with consistent patterns, good use of parameterized queries, and clean separation between connection management, caching, and domain models. The codebase is mature and in good shape. This review builds on the previous review (2026-04-20) and re-evaluates all findings.

---

## Issues

### 1. `RowColumn::get_column` panics on missing columns or NULL values — **Design Flaw** ⚠️

- **File:** `db/src/tiberius/row_column.rs`, lines 20–23
- **Problem:** The `get_column` implementations use `.unwrap().unwrap()`, which will panic if a column is missing or contains a SQL NULL. While this is acceptable for columns known to be NOT NULL, a schema change or unexpected NULL will cause a runtime panic with no context about which column failed.
- **Suggested fix:** Use `.expect("column <name>")` or a helper that includes the column name in the panic message to aid debugging.

### 2. `RowColumn<DateTime<Utc>>` silently returns epoch on error — **Design Flaw** ⚠️

- **File:** `db/src/tiberius/row_column.rs`, lines 48–57
- **Problem:** The `RowColumn<DateTime<Utc>>` implementation returns `DateTime::from_timestamp(0, 0).unwrap()` (Unix epoch) when `try_get` fails. This silently masks errors — the caller receives a valid-looking timestamp instead of an error.
- **Suggested fix:** Either panic with context (consistent with other `get_column` impls) or propagate the error.

### 3. `TryRowColumn` implementations silently swallow type conversion errors — **Design Flaw** ⚠️

- **File:** `db/src/tiberius/row_column.rs`, lines 28–36
- **Problem:** The macro-generated `TryRowColumn` implementations use `unwrap_or_default()` on the outer `Result`, meaning a column type mismatch error is silently treated as `None`. Only column-not-found and NULL should return `None`.
- **Suggested fix:** Distinguish between "column not found / NULL" (return `None`) and "type conversion error" (log a warning or propagate).

### 4. `TryRowColumn<String>` treats empty strings as `None` — **Minor**

- **File:** `db/src/tiberius/row_column.rs`, lines 59–72
- **Problem:** Empty strings are returned as `None` rather than `Some("")`. This conflates "no value" with "empty value", which may cause subtle bugs if the distinction matters.
- **Suggested fix:** Return `Some("".to_string())` for empty strings, or document this behavior prominently.

### 5. ~~`HeatResult::points` can underflow for `rank > 5`~~ ✅ FIXED

- **File:** `db/src/aquarius/model/heat_result.rs`, line 38
- **Fix:** Added a guard `rank > 0 && rank <= 5` so the subtraction `5 - rank` only executes when it is safe. Ranks above 5 (or rank 0) now correctly yield 0 points instead of underflowing.

### 6. `Statistics::query` holds mutable borrow on `client` across `join!` — **Minor Efficiency** 💡

- **File:** `db/src/aquarius/model/statistics.rs`, lines 184–189
- **Problem:** `join!` is used with `query.query(&mut client)` alongside `Statistics::query_oldest(...)` calls that also acquire their own pool connections. This means 3 connections are held simultaneously for one logical operation.
- **Suggested fix:** Sequence the main query before the concurrent oldest-athlete queries to release the connection earlier, reducing pool pressure.

### 7. ~~Duplicated SQL aggregation subqueries in `Club`~~ ✅ FIXED

- **File:** `db/src/aquarius/model/club.rs`
- **Fix:** Extracted the three duplicated aggregation subqueries (Participations_Count, Athletes_Female_Count, Athletes_Male_Count) into a shared `Club::aggregation_subqueries(alias)` helper method, called by both `query_clubs_participating_regatta` and `query_club_with_aggregations`.

### 8. `TimeStrip::add_start` and `add_finish` are nearly identical — **Code Duplication** 📋

- **File:** `db/src/timekeeper/timestrip.rs`, lines 41–63
- **Problem:** `add_start` and `add_finish` differ only in the `Split` variant passed. This is code duplication.
- **Suggested fix:** Extract a private `add_timestamp(split: Split, time: Option<DateTime<Utc>>)` method.

### 9. `DbError` has two overlapping cache error variants — **Design** 💡

- **File:** `db/src/error.rs`, lines 17–20
- **Problem:** `DbError::Cache(String)` and `DbError::CacheError(#[from] CacheError)` serve similar purposes. The `Cache(String)` variant is used in `cache.rs` to wrap computation errors via `format!("Computation failed: {}", e)`, losing the original error type. Having two cache variants with identical display messages (`"Cache error: {0}"`) is confusing.
- **Suggested fix:** Consider unifying into a single variant, or rename them to distinguish their purpose clearly (e.g., `CacheComputation(String)` vs `CacheDriver(CacheError)`).

### 10. `compute_if_missing` loses original error type information — **Minor** 💡

- **File:** `db/src/cache.rs`, lines 116–118
- **Problem:** The `compute_if_missing` and `compute_if_missing_opt` methods convert computation errors to strings via `DbError::Cache(format!("Computation failed: {}", e))`. This loses the original error type (which is typically already a `DbError`), making it harder to match on specific error variants upstream.
- **Suggested fix:** Since `F`'s error type is bounded by `Display` rather than `Into<DbError>`, consider tightening the bound to `Into<DbError>` so the original error is preserved, or keep the current approach and document that error context is intentionally simplified at the cache boundary.

### 11. `get_visible_notifications` hardcodes `force_cache: false` — **Minor** 💡

- **File:** `db/src/aquarius.rs`, line 299
- **Problem:** `get_visible_notifications` always passes `false` for `force`, unlike other methods that accept `force_cache` from the caller. This means the caller cannot force a cache refresh for visible notifications.
- **Suggested fix:** Add a `force_cache: bool` parameter to be consistent with other methods.

### 12. Cache `max_cost` calculation uses `mem::size_of` (stack size only) — **Minor Inaccuracy** 💡

- **File:** `db/src/cache/config.rs`, lines 38–68
- **Problem:** `max_cost` is calculated as `mem::size_of::<T>() * MAX_COUNT`, but `mem::size_of` only accounts for stack size, not heap allocations (e.g., `String` fields). Meanwhile, `CacheCost` implementations correctly include heap size. This mismatch means the configured `max_cost` may be significantly smaller than the actual memory usage of cached entries, potentially causing premature eviction.
- **Suggested fix:** Use a more realistic estimate that accounts for typical heap allocations per entry, or make `max_cost` configurable/tunable.

### 13. ~~`Aquarius::get_athlete` cache key ignores `regatta_id`~~ ✅ FIXED

- **Files:** `db/src/aquarius.rs`, `db/src/cache.rs`
- **Fix:** Changed the `athlete` cache from `Cache<i32, Athlete>` to `Cache<(i32, i32), Athlete>` and updated `get_athlete` to use the composite key `(regatta_id, athlete_id)`, consistent with other composite-key caches like `athlete_entries`.

### 14. Magic number `64` used throughout for "final round" — **Maintainability** 💡

- **Files:** Multiple model files (`entry.rs` line 114, `athlete.rs` line 63, `crew.rs` implicit, `heat_entry.rs`, `statistics.rs`)
- **Problem:** The value `64` appears repeatedly as a magic number representing the "final round". While consistent, it lacks documentation and a named constant.
- **Suggested fix:** Define a named constant (e.g., `const ROUND_FINAL: i16 = 64;`) in the model module and use it throughout.

---

## Previously Fixed Issues ✅

- ~~`Timestamp::persist` redundantly calls `.to_string()` on `format!()`~~ — FIXED
- ~~`Score::calculate` uses manual rank counter instead of `enumerate`~~ — FIXED

---

## Positive Observations

- **Parameterized queries throughout:** All SQL queries use `Query::new()` with `.bind()` for parameters — no string interpolation of user input. ✅
- **Consistent patterns:** All model types follow `From<&Row>` + `HeapSize`/`CacheCost`, making the codebase predictable. ✅
- **Good use of concurrent queries:** `join!`, `join3`, and `join_all` are used effectively to parallelize independent DB queries. ✅
- **Well-designed cache layer:** The `Cache<K, V>` abstraction with `compute_if_missing` provides a clean cache-aside pattern with TTL and cost-based eviction. ✅
- **Clean error type:** `DbError` using `thiserror` with `#[from]` conversions is idiomatic. ✅
- **Column name constants:** Model files define column names as `const` strings, reducing typo risk. ✅
- **Existing test coverage:** `flags_scraper` has a unit test validating the HTML parsing logic. ✅
- **`OnceLock` for lazy globals:** `ClubFlag` and `TiberiusPool` use `OnceLock` for safe lazy initialization. ✅
- **Clean clippy output:** No warnings from `cargo clippy`. ✅
- **Thread-safe pool initialization:** `TiberiusPool::init` uses a double-check locking pattern with `OnceLock` + `Mutex` to ensure exactly-once initialization. ✅
- **Effective user pool management:** `UserPoolManager` uses `RwLock` with double-checked locking for per-user connection pool caching. ✅
- **Good cache invalidation strategy:** Write operations (create/update/delete notification) properly invalidate the corresponding cache entries. ✅
- **`TryToEntity` trait:** Clean abstraction for optional entity construction from rows where the entity's columns may not be present. ✅