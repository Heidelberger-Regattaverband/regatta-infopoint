## Review of the `db` module

### Overall Impression
Well-structured crate with clear separation of concerns: caching (`cache`), DB access (`tiberius`), domain models (`aquarius/model`), and timekeeping. The code is generally clean and idiomatic. Below are specific findings and improvement suggestions.

---

### 1. **`error.rs` — Duplicate cache error variants**

`DbError` has both `Cache(String)` and `CacheError(#[from] CacheError)` with **identical `#[error]` messages**. This is confusing and makes matching ambiguous.

**Suggestion:** Remove `Cache(String)` and use `Custom(String)` for ad-hoc messages, or rename the variants to have distinct display messages.

---

### 2. **`lib.rs` — Naming collision with `tiberius`**

```rust
pub mod tiberius;
pub use ::tiberius as tiberius_client;
```

Re-exporting the external `tiberius` crate while also having a `tiberius` module is confusing for consumers. Consider renaming your module (e.g., `pool` or `db_pool`) or the re-export.

---

### 3. **`row_column.rs` — Panics on missing/NULL columns**

All `RowColumn` impls use `.unwrap().unwrap()`, which will **panic** on missing columns or NULL values. This is a runtime crash risk.

**Suggestion:** Return `Result<T, DbError>` instead of `T`, or at minimum document that these are only safe on known-not-null columns. A macro could reduce boilerplate:

```rust
macro_rules! impl_row_column {
    ($($ty:ty),*) => { $(
        impl RowColumn<$ty> for Row {
            fn get_column(&self, col_name: &str) -> $ty {
                self.try_get::<$ty, _>(col_name).unwrap().unwrap()
            }
        }
    )* };
}
impl_row_column!(bool, u8, i16, i32, f32, f64, NaiveDateTime, NaiveDate);
```

---

### 5. **`cache.rs` — `compute_if_missing` / `compute_if_missing_opt` duplication**

These two methods are nearly identical. Consider unifying them, e.g., by always working with `Option<V>` internally, or using a helper trait.

---

### 7. **`aquarius.rs` — Global singleton `TiberiusPool::instance()` used everywhere**

Every query closure calls `TiberiusPool::instance()` and `.get().await?`. This tight coupling to a global singleton makes testing impossible and violates dependency injection principles.

**Suggestion:** Store a reference/`Arc` to `TiberiusPool` in `Aquarius` and pass it through, enabling unit testing with mock pools.

---

### 9. **`pool.rs` — `new()` panics on failure**

`TiberiusPool::new` calls `.expect("Failed to create Tiberius connection pool")`. This should return `Result<Self, DbError>` to allow graceful error handling by callers.

---

### 10. **`user_pool.rs` — Minor issues**

- Multiple `#[allow(dead_code)]` hints that these methods may be unnecessary or that the API surface needs pruning.
- No pool eviction/TTL strategy — pools accumulate indefinitely in memory.

---

### 12. **Missing `#[must_use]` annotations**

Public methods returning values (e.g., `get_cache_stats`, `state`) should have `#[must_use]` to prevent accidental discarding of results.

---

### 13. **No tests**

The `[dev-dependencies]` includes `tokio-shared-rt` but there are no test files visible. Adding unit tests — especially for `Cache`, `CacheCost`, and `RowColumn` — would improve reliability.

---

### Summary of Priority Improvements

| Priority | Issue | Impact |
|----------|-------|--------|
| 🔴 High | `RowColumn` panics on NULL/missing columns | Runtime crashes |
| 🔴 High | Global singleton coupling in `Aquarius` | Untestable |
| 🟡 Medium | `CacheCost` underestimates heap usage | Poor cache eviction |
| 🟡 Medium | `TiberiusPool::new` panics | No graceful error handling |
| 🟡 Medium | Duplicate `DbError` cache variants | Confusing API |
| 🟢 Low | Missing tests | Reliability |
