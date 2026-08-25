# Code Review: `db` Crate

**Date:** 2026-04-26  
**Scope:** All source files in `db/src/`  
**Clippy:** ✅ Clean (no warnings)

---

## Summary

The `db` crate is well-structured with consistent patterns, good use of parameterized queries, and clean separation between connection management, caching, and domain models. The codebase is mature and in good shape. This review builds on the previous review (2026-04-24) and re-evaluates all findings.

---

## Issues

### 3. `TryRowColumn` implementations silently swallow type conversion errors — **Design Flaw** ⚠️

- **File:** `db/src/tiberius/row_column.rs`, lines 28–36
- **Problem:** The macro-generated `TryRowColumn` implementations use `unwrap_or_default()` on the outer `Result`, meaning a column type mismatch error is silently treated as `None`. Only column-not-found and NULL should return `None`.
- **Suggested fix:** Distinguish between "column not found / NULL" (return `None`) and "type conversion error" (log a warning or propagate).

### 9. `get_visible_notifications` hardcodes `force_cache: false` — **Minor** 💡

- **File:** `db/src/aquarius.rs`, line 300
- **Problem:** `get_visible_notifications` always passes `false` for `force`, unlike other methods that accept `force_cache` from the caller. This means the caller cannot force a cache refresh for visible notifications.
- **Suggested fix:** Add a `force_cache: bool` parameter to be consistent with other methods.

### 10. Magic number `64` used throughout for "final round" — **Maintainability** 💡

- **Files:** Multiple model files (`entry.rs` line 107, `athlete.rs` line 52, `crew.rs` implicit, `heat_entry.rs`, `statistics.rs`, `score.rs` line 53)
- **Problem:** The value `64` appears repeatedly as a magic number representing the "final round". While consistent, it lacks documentation and a named constant.
- **Suggested fix:** Define a named constant (e.g., `const ROUND_FINAL: i16 = 64;`) in the model module and use it throughout.

### 13. `Regatta::query_active_regatta` returns first regatta, not necessarily "active" — **Semantic** 💡

- **File:** `db/src/aquarius/model/regatta.rs`, lines 63–71
- **Problem:** The method is named `query_active_regatta` but the SQL simply selects the regatta with the most recent start date (`ORDER BY e.Event_StartDate DESC, e.Event_ID DESC`). There is no explicit "active" flag in the query. If multiple regattas exist, a future regatta could be returned if its start date is later.
- **Suggested fix:** Document this behavior clearly, or add an explicit check against the current date range.

### 14. `Score::calculate` query uses `Club_ID = Athlet_Club_ID_FK` — **Potential Semantic Issue** 💡

- **File:** `db/src/aquarius/model/score.rs`, lines 33–57
- **Problem:** The scoring query joins `Club ON Club_ID = Athlet_Club_ID_FK`, which groups scores by the athlete's club. However, entries have their own `Entry_OwnerClub_ID_FK` which represents the registering club. For athletes competing under a racing community (different from their home club), scores might be attributed to the athlete's home club rather than the entry's registering club. This may or may not be intentional.
- **Suggested fix:** Verify this is the desired scoring semantics. If scores should follow the entry's registering club, use `Entry_OwnerClub_ID_FK` instead.

### 16. `HeatEntry::query_entries_of_heat` has complex SQL filter with implicit assumptions — **Minor** 💡

- **File:** `db/src/aquarius/model/heat_entry.rs`, lines 58–69
- **Problem:** The WHERE clause `((Result_SplitNr = 64 AND Comp_State >=4) OR (Result_SplitNr = 0 AND Comp_State < 3) OR (Comp_State < 2 AND Result_SplitNr IS NULL))` encodes business logic about heat states with magic numbers. This is difficult to understand and maintain.
- **Suggested fix:** Add a comment explaining each condition branch, or extract the conditions into named constants/helper functions.

---

## New Issues (2026-08-24)

### N3. `Timestamp.bib` typed as `Option<u8>` — overflow for large regattas — **MEDIUM**

**File:** `db/src/timekeeper/timestamp.rs`, line 32

`Timestamp.bib: Option<u8>` can represent bibs 0–255 only. `Entry.bib` is correctly `Option<i16>`. Regattas with more than 255 participants will silently lose bib assignments on read (the `TryRowColumn<u8>` call will fail for values > 255) and send truncated values on write.

**Suggested fix:** Change `bib: Option<u8>` to `bib: Option<i16>` and update related `TryRowColumn` accesses.

---

### N4. `UserPoolManager` pools are never reclaimed — unbounded memory and credential retention — **MEDIUM**

**File:** `db/src/tiberius/user_pool.rs`, lines 56–74

`remove_pool`, `clear_all`, and `pool_count` are all `#[allow(dead_code)]`, meaning every authenticated user gets a persistent `TiberiusPool` that holds live DB connections and plaintext credentials indefinitely. Memory and connection count grow without bound as users authenticate.

**Suggested fix:** Wire `remove_pool` to the logout/session-expiry path. Consider a background eviction task for idle pools.

---

### N5. `Debug` derive on pool types risks credential exposure in logs — **MEDIUM**

**File:** `db/src/tiberius/connection.rs`, line 12; `db/src/tiberius/pool.rs`, line 16

`TiberiusConnectionManager` (which contains `tiberius::Config` with auth credentials) and `TiberiusPool` both derive `Debug`. Any `{:?}` log of these types may expose passwords if tiberius's `Config` does not redact them.

**Suggested fix:** Implement `Debug` manually for `TiberiusConnectionManager`, printing only the host and omitting credentials:
```rust
impl fmt::Debug for TiberiusConnectionManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TiberiusConnectionManager")
            .field("host", &self.config.get_addr())
            .finish_non_exhaustive()
    }
}
```

---

### N6. `Split::from(u8)` silently defaults unknown values to `Split::Start` — **LOW/MEDIUM**

**File:** `db/src/timekeeper/timestamp.rs`, lines 163–170

Unknown split codes (e.g., a 500m intermediate split) are silently classified as `Start`, potentially corrupting timestrip calculations.

**Suggested fix:** Add a `tracing::warn!` for the fallback case, or return `Option<Split>` and handle `None` explicitly.

---

### N7. Integer underflow if `bib == 0` in `ClubConflictRace::query_club_conflicts` — **LOW**

**File:** `db/src/aquarius/model/problems.rs`, line 151

```rust
let heat_number = ((entry.bib as usize - 1) / LANES) + 1;
```

If `entry.bib == 0`, this underflows to `usize::MAX` in release builds.

**Suggested fix:** Add `AND e.Entry_Bib > 0` to the SQL, or guard: `if entry.bib == 0 { continue; }`.

---

### N8. N+1 parallel queries in `execute_query` can saturate the connection pool — **LOW**

**File:** `db/src/aquarius/model/entry.rs`, lines 198–233

For a 200-entry result, up to 400 simultaneous pool connections are requested via `join_all`. Under concurrent API requests this multiplies.

**Suggested fix:** Chunk `join_all` into batches, or restructure into a single JOIN query.

---

### N9. Negative delta underflow in `HeatEntry::query_entries_of_heat` — **LOW**

**File:** `db/src/aquarius/model/heat_entry.rs`, lines 123–128

```rust
let delta = result.net_time - first_net_time;
let duration = Duration::from_millis(delta as u64);
```

If `result.net_time < first_net_time`, casting a negative `i32` to `u64` wraps to a huge value, producing a nonsensical delta string.

**Suggested fix:** Guard with `if delta > 0 { ... }`.

---

### N10. `UserPoolManager::create_pool` hardcodes pool sizes — **LOW**

**File:** `db/src/tiberius/user_pool.rs`, line 50

`TiberiusPool::new(config, 5, 1)` is hardcoded. With many authenticated admin users, total connections grow unbounded at 5× per user.

**Suggested fix:** Accept `max_size` and `min_idle` as parameters to `UserPoolManager::new`.

---
