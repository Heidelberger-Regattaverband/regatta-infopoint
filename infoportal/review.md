# Infoportal Module Review

## Overview
The `infoportal` crate is an actix-web HTTP/HTTPS server serving a regatta information portal. It provides a REST API backed by a MS-SQL database (via Tiberius), WebSocket endpoints for real-time timekeeping and monitoring, Swagger/OpenAPI documentation, and static file serving for the UI.

## Architecture
- **`main.rs`** – Initializes the DB pool and starts the server. Includes integration tests.
- **`config.rs`** – Environment-based configuration with validation, constants, and typed error handling. Uses `SecretString` for the DB password.
- **`auth.rs`** – Credential verification by attempting a DB login; centralized admin role determination via `Scope::from_username`.
- **`peak_alloc.rs`** – Custom global allocator tracking current and peak memory usage.
- **`http/server.rs`** – Server setup: TLS, rate limiting, session/identity middleware, Prometheus metrics, static files.
- **`http/rest_api.rs`** – REST API route registration, `ApiError` newtype for DB error handling, shared handler utilities.
- **`http/rest_api/*.rs`** – Individual endpoint modules (race, club, athlete, notification, timekeeping, monitoring, authentication, misc).
- **`http/api_doc.rs`** – OpenAPI/Swagger UI configuration.
- **`http/monitoring.rs`** – Monitoring data model (DB connections, system info, cache stats, app memory).

## Strengths
1. **Well-structured configuration** – `config.rs` is thorough: typed parsing, required vs. optional env vars, validation with meaningful errors, and constants separated into a submodule.
2. **Good use of middleware** – Rate limiting, session management, identity/auth, Prometheus metrics are all cleanly layered.
3. **OpenAPI integration** – All endpoints are annotated with `utoipa` and exposed via Swagger UI.
4. **Consistent error handling** – `ApiError` newtype wraps `DbError` with logging and proper `ResponseError` implementation.
5. **Custom allocator** – `PeakAlloc` is a clean, minimal implementation for memory monitoring.
6. **WebSocket architecture** – Timekeeping and monitoring use actix actors with proper heartbeat/timeout handling.
7. **Consistent auth guard** – Auth-required endpoints use required `Identity` parameter; public endpoints use `Option<Identity>`.
8. **Centralized admin check** – `Scope::from_username` provides a single source of truth for admin role determination.

## Issues and Suggestions

### Bugs / Correctness

1. **`auth.rs:84` – Unnecessary `.to_string()` after `.to_lowercase()`**
   ```rust
   let username = credentials.username.trim().to_lowercase().to_string();
   ```
   `.to_lowercase()` already returns a `String`, so the final `.to_string()` is redundant. Use `let username = credentials.username.trim().to_lowercase();`.

2. **`rest_api.rs:22-24` – Unnecessary imports**
   `std::fmt`, `std::fmt::Display`, `std::fmt::Formatter` are imported but the `Display` impl directly uses `std::fmt::Formatter` qualified. Consider using either the qualified or imported form consistently. Also `ErrorInternalServerError` is imported but only used in `get_user_pool` — verify it's still needed.

### Security

3. **`auth.rs` – Authentication via raw DB connection attempt**
   Using a SQL Server login attempt as the authentication mechanism is fragile. Failed logins may trigger SQL Server lockout policies. Consider at minimum rate-limiting login attempts per user.

4. **`server.rs:65` – Session secret key generated at startup**
   `Key::generate()` means all sessions are invalidated on every restart. For production, consider persisting the key or deriving it from a stable secret.

### Design / Maintainability

5. **`server.rs:63` – `unwrap()` on `create_app_data()`**
   If the DB is unreachable at startup, this panics without a clear message. Consider propagating the error with context.

6. **`server.rs:157-163` – Dead `wrap_fn` middleware**
   The `wrap_fn` closure contains only commented-out print statements. It should be removed as it adds overhead for no purpose.

7. **`timekeeping.rs` – Mixing `std::sync::mpsc` with async/actix**
   The timekeeping actor uses `std::sync::mpsc` and `std::thread::spawn` for Aquarius event receiving. This blocks a system thread. Consider using `tokio::sync::mpsc` with `ctx.add_stream()` instead.

8. **`timekeeping.rs:164` – `unwrap()` on `AquariusClient::new`**
   If the Aquarius client can't be created, the actor panics. Should return an error to the WebSocket client.

9. **`timekeeping.rs:168` – `unwrap()` on `TimeStrip::load`**
   Same issue — panics if the timestrip can't be loaded from the DB.

10. **`monitoring.rs:68` – `std::thread::sleep` in monitoring data collection**
    `get_system()` calls `std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL)` which blocks the current thread. In the monitoring WebSocket actor this runs on every heartbeat interval (2 seconds), blocking an actix worker thread. Consider collecting CPU metrics asynchronously or caching them.

11. **`rest_api.rs:25` – Unused import `Arc`**
    `use ::std::sync::Arc;` may be unused in `rest_api.rs` itself (it's used in `get_user_pool` return type). Verify with `cargo clippy`.

12. **Integration tests in `main.rs` require a live database**
    Tests (`test_get_regattas`, `test_get_heats`) connect to a real DB, making them integration tests that can't run in CI without infrastructure. Consider separating unit and integration tests.

### Minor / Cosmetic

13. **`config.rs:16` – Stale doc comment** mentions `Config::get()` which doesn't exist.

14. **`monitoring.rs:50` – `Disks::new_with_refreshed_list()` called on every monitoring update**
    Disk information rarely changes. Consider caching it or refreshing less frequently.

15. **`rest_api.rs` – Handlers in root module vs sub-modules**
    `get_filters`, `get_active_regatta`, `get_heats`, `get_heat` live in the root `rest_api.rs` while similar handlers are in sub-modules. Consider moving them to dedicated sub-modules (e.g., `regatta.rs`, `heat.rs`) for consistency.

16. **`monitoring.rs` (actor) – Local `HEARTBEAT_INTERVAL` and `CLIENT_TIMEOUT` constants**
    Both `monitoring.rs` and `timekeeping.rs` define their own heartbeat/timeout constants with the same names but different values. Consider extracting shared defaults or documenting the intentional differences.

## Summary
The module is well-organized with good separation of concerns. Previous review items have been addressed: error handling uses `ApiError`, admin checks are centralized, auth guards are consistent, `INFOPORTAL_V2` string is fixed, `db_password` uses `SecretString`, and username trimming is correct. Remaining areas for improvement are: removing dead code (`wrap_fn`), avoiding blocking calls in async contexts (monitoring sleep, `std::sync::mpsc`), handling `unwrap()` panics in actor initialization, and the session key generation strategy.