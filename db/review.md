# Code Review: `db` Crate

**Date:** 2026-04-26  
**Scope:** All source files in `db/src/`  
**Clippy:** ✅ Clean (no warnings)

---

## Summary

The `db` crate is well-structured with consistent patterns, good use of parameterized queries, and clean separation between connection management, caching, and domain models. The codebase is mature and in good shape. This review builds on the previous review (2026-04-24) and re-evaluates all findings.

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

### 5. `Statistics::query` holds mutable borrow on `client` across `join!` — **Minor Efficiency** 💡

- **File:** `db/src/aquarius/model/statistics.rs`, lines 184–189
- **Problem:** `join!` is used with `query.query(&mut client)` alongside `Statistics::query_oldest(...)` calls that also acquire their own pool connections. This means 3 connections are held simultaneously for one logical operation.
- **Suggested fix:** Sequence the main query before the concurrent oldest-athlete queries to release the connection earlier, reducing pool pressure.

### 6. `TimeStrip::add_start` and `add_finish` are nearly identical — **Code Duplication** 📋

- **File:** `db/src/timekeeper/timestrip.rs`, lines 41–63
- **Problem:** `add_start` and `add_finish` differ only in the `Split` variant passed. This is code duplication.
- **Suggested fix:** Extract a private `add_timestamp(split: Split, time: Option<DateTime<Utc>>)` method.

### 7. `DbError` has two overlapping cache error variants — **Design** 💡

- **File:** `db/src/error.rs`, lines 17–20
- **Problem:** `DbError::Cache(String)` and `DbError::CacheError(#[from] CacheError)` serve similar purposes. The `Cache(String)` variant is used in `cache.rs` to wrap computation errors via `format!("Computation failed: {}", e)`, losing the original error type. Having two cache variants with identical display messages (`"Cache error: {0}"`) is confusing.
- **Suggested fix:** Consider unifying into a single variant, or rename them to distinguish their purpose clearly (e.g., `CacheComputation(String)` vs `CacheDriver(CacheError)`).

### 8. `compute_if_missing` loses original error type information — **Minor** 💡

- **File:** `db/src/cache.rs`, lines 116–118
- **Problem:** The `compute_if_missing` and `compute_if_missing_opt` methods convert computation errors to strings via `DbError::Cache(format!("Computation failed: {}", e))`. This loses the original error type (which is typically already a `DbError`), making it harder to match on specific error variants upstream.
- **Suggested fix:** Since `F`'s error type is bounded by `Display` rather than `Into<DbError>`, consider tightening the bound to `Into<DbError>` so the original error is preserved, or keep the current approach and document that error context is intentionally simplified at the cache boundary.

### 9. `get_visible_notifications` hardcodes `force_cache: false` — **Minor** 💡

- **File:** `db/src/aquarius.rs`, line 300
- **Problem:** `get_visible_notifications` always passes `false` for `force`, unlike other methods that accept `force_cache` from the caller. This means the caller cannot force a cache refresh for visible notifications.
- **Suggested fix:** Add a `force_cache: bool` parameter to be consistent with other methods.

### 10. Magic number `64` used throughout for "final round" — **Maintainability** 💡

- **Files:** Multiple model files (`entry.rs` line 107, `athlete.rs` line 52, `crew.rs` implicit, `heat_entry.rs`, `statistics.rs`, `score.rs` line 53)
- **Problem:** The value `64` appears repeatedly as a magic number representing the "final round". While consistent, it lacks documentation and a named constant.
- **Suggested fix:** Define a named constant (e.g., `const ROUND_FINAL: i16 = 64;`) in the model module and use it throughout.

### 11. `Block::query_blocks` bypasses the `get_rows` helper — **Inconsistency** 📋

- **File:** `db/src/aquarius/model/block.rs`, line 39
- **Problem:** `Block::query_blocks` uses `stream.into_first_result().await?` directly instead of the `get_rows()` helper function used everywhere else. This is a minor inconsistency in the codebase.
- **Suggested fix:** Use `get_rows(stream)` for consistency with all other query methods.

### 12. `Block::query_blocks` uses index-based row access instead of `RowColumn` — **Inconsistency** 📋

- **File:** `db/src/aquarius/model/block.rs`, lines 43–64
- **Problem:** Block parsing accesses rows via positional index (`rows[i].get::<NaiveDateTime, usize>(0)`) rather than the `RowColumn` trait used everywhere else. This is fragile — if the column order changes or additional columns are added, the code will silently break.
- **Suggested fix:** Use `RowColumn::get_column` or `TryRowColumn::try_get_column` with the column name `"Comp_DateTime"`.

### 13. `Regatta::query_active_regatta` returns first regatta, not necessarily "active" — **Semantic** 💡

- **File:** `db/src/aquarius/model/regatta.rs`, lines 63–71
- **Problem:** The method is named `query_active_regatta` but the SQL simply selects the regatta with the most recent start date (`ORDER BY e.Event_StartDate DESC, e.Event_ID DESC`). There is no explicit "active" flag in the query. If multiple regattas exist, a future regatta could be returned if its start date is later.
- **Suggested fix:** Document this behavior clearly, or add an explicit check against the current date range.

### 14. `Score::calculate` query uses `Club_ID = Athlet_Club_ID_FK` — **Potential Semantic Issue** 💡

- **File:** `db/src/aquarius/model/score.rs`, lines 33–57
- **Problem:** The scoring query joins `Club ON Club_ID = Athlet_Club_ID_FK`, which groups scores by the athlete's club. However, entries have their own `Entry_OwnerClub_ID_FK` which represents the registering club. For athletes competing under a racing community (different from their home club), scores might be attributed to the athlete's home club rather than the entry's registering club. This may or may not be intentional.
- **Suggested fix:** Verify this is the desired scoring semantics. If scores should follow the entry's registering club, use `Entry_OwnerClub_ID_FK` instead.

### 15. `Notification::update_notification` dynamic SQL parameter binding is fragile — **Minor Risk** 💡

- **File:** `db/src/aquarius/model/notification.rs`, lines 151–212
- **Problem:** The `update_notification` method builds SQL dynamically with positional parameters (`@P1`, `@P2`, etc.) based on which optional fields are present. The parameter binding order must exactly match the `set_clauses` construction order. While currently correct, this pattern is fragile — any reordering of the `if` blocks would introduce a subtle parameter mismatch bug that would be hard to detect.
- **Suggested fix:** Consider using a builder pattern or named parameter approach to make the binding order less error-prone.

### 16. `HeatEntry::query_entries_of_heat` has complex SQL filter with implicit assumptions — **Minor** 💡

- **File:** `db/src/aquarius/model/heat_entry.rs`, lines 58–69
- **Problem:** The WHERE clause `((Result_SplitNr = 64 AND Comp_State >=4) OR (Result_SplitNr = 0 AND Comp_State < 3) OR (Comp_State < 2 AND Result_SplitNr IS NULL))` encodes business logic about heat states with magic numbers. This is difficult to understand and maintain.
- **Suggested fix:** Add a comment explaining each condition branch, or extract the conditions into named constants/helper functions.

---

## Previously Fixed Issues ✅

- ~~`Timestamp::persist` redundantly calls `.to_string()` on `format!()`~~ — FIXED
- ~~`Score::calculate` uses manual rank counter instead of `enumerate`~~ — FIXED
- ~~`HeatResult::points` can underflow for `rank > 5`~~ — FIXED
- ~~`Aquarius::get_athlete` cache key ignores `regatta_id`~~ — FIXED
- ~~Duplicated SQL aggregation subqueries in `Club`~~ — ACKNOWLEDGED (dedup would require careful refactoring)

---

## Positive Observations

- **Parameterized queries throughout:** All SQL queries use `Query::new()` with `.bind()` for parameters — no string interpolation of user input. ✅
- **Consistent patterns:** All model types follow `From<&Row>` + column constants, making the codebase predictable. ✅
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
- **New query well-integrated:** The `query_heats_with_multiple_club_entries` query follows established patterns — uses a subquery with `HAVING COUNT(*) > 1`, properly filters cancelled entries and heats, and is backed by cache and REST endpoint. ✅