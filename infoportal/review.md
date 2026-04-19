# Infoportal Module Review

## Overview
The `infoportal` crate is an actix-web HTTP/HTTPS server serving a regatta information portal. It provides a REST API backed by a MS-SQL database (via Tiberius), WebSocket endpoints for real-time timekeeping and monitoring, Swagger/OpenAPI documentation, and static file serving for the UI.

## Architecture
- **`main.rs`** – Initializes the DB pool and starts the server. Includes integration tests.
- **`config.rs`** – Environment-based configuration with validation, constants, and typed error handling.
- **`auth.rs`** – Credential verification by attempting a DB login; role determination.
- **`peak_alloc.rs`** – Custom global allocator tracking current and peak memory usage.
- **`http/server.rs`** – Server setup: TLS, rate limiting, session/identity middleware, Prometheus metrics, static files.
- **`http/rest_api.rs`** – REST API route registration and shared handler utilities.
- **`http/rest_api/*.rs`** – Individual endpoint modules (race, club, athlete, notification, timekeeping, monitoring, authentication, misc).
- **`http/api_doc.rs`** – OpenAPI/Swagger UI configuration.
- **`http/monitoring.rs`** – Monitoring data model (DB connections, system info, cache stats, app memory).

## Strengths
1. **Well-structured configuration** – `config.rs` is thorough: typed parsing, required vs. optional env vars, validation with meaningful errors, and constants separated into a submodule.
2. **Good use of middleware** – Rate limiting, session management, identity/auth, Prometheus metrics are all cleanly layered.
3. **OpenAPI integration** – All endpoints are annotated with `utoipa` and exposed via Swagger UI.
4. **Consistent error handling** – REST handlers uniformly map errors to `ErrorInternalServerError` with logging.
5. **Custom allocator** – `PeakAlloc` is a clean, minimal implementation for memory monitoring.
6. **WebSocket architecture** – Timekeeping and monitoring use actix actors with proper heartbeat/timeout handling.

## Issues and Suggestions

### Bugs / Correctness

2. **`auth.rs:74-75` – Unnecessary mutable default + clone_into**
   ```rust
   let mut username: String = Default::default();
   credentials.username.trim().clone_into(&mut username);
   ```
   Simpler: `let username = credentials.username.trim().to_string();`

3. **`auth.rs:92` – Hardcoded admin check against `"sa"`**
   The admin role is determined by comparing the username to `"sa"`. This is duplicated in `authentication.rs:85` where `"sa" | "admin"` is checked. These should be consistent and ideally configurable.

4. **`auth.rs:98` – Uses untrimmed username for the `User` struct**
   Line 75 trims the username into a local variable, but line 98 uses `credentials.username.clone()` (untrimmed). Should use the trimmed `username` variable instead.

5. **`server.rs:157-163` – Dead wrap_fn middleware**
   The `wrap_fn` closure contains only commented-out print statements. It should be removed as it adds overhead for no purpose.

### Security

6. **`auth.rs` – Authentication via raw DB connection attempt**
   Using a SQL Server login attempt as the authentication mechanism is fragile. Failed logins may trigger SQL Server lockout policies. Consider at minimum rate-limiting login attempts per user.

7. **`server.rs:65` – Session secret key generated at startup**
   `Key::generate()` means all sessions are invalidated on every restart. For production, consider persisting the key or deriving it from a stable secret.

### Design / Maintainability

9. **`server.rs:63` – `unwrap()` on `create_app_data()`**
   If the DB is unreachable at startup, this panics without a clear message. Consider propagating the error with context.

10. **`timekeeping.rs` – Mixing `std::sync::mpsc` with async/actix**
    The timekeeping actor uses `std::sync::mpsc` and `std::thread::spawn` for Aquarius event receiving. This blocks a system thread. Consider using `tokio::sync::mpsc` with `ctx.add_stream()` instead.

11. **`timekeeping.rs:161` – `unwrap()` on `AquariusClient::new`**
    If the Aquarius client can't be created, the actor panics. Should return an error to the WebSocket client.

12. **`timekeeping.rs:165` – `unwrap()` on `TimeStrip::load`**
    Same issue — panics if the timestrip can't be loaded from the DB.

13. **`monitoring.rs:68` – `std::thread::sleep` in monitoring data collection**
    `get_system()` calls `std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL)` which blocks the current thread. In the monitoring WebSocket actor this runs on every heartbeat interval (2 seconds), blocking an actix worker thread. Consider collecting CPU metrics asynchronously or caching them.

14. **`rest_api.rs:21` – Unused import `Arc`**
    `use ::std::sync::Arc;` appears unused in `rest_api.rs` (it's used in the `get_user_pool` return type but via the function signature). Verify if needed.

15. **`rest_api.rs:30-32` – Constants `HEARTBEAT_INTERVAL` and `CLIENT_TIMEOUT` in wrong module**
    These WebSocket constants are defined in the REST API root module but only used by the WebSocket sub-modules. They should live closer to their usage or in a shared `ws` module.

16. **Repetitive error handling pattern across handlers**
    Every handler repeats `.map_err(|err| { error!("{err}"); ErrorInternalServerError(err) })`. Consider an `Into<actix_web::Error>` impl for `DbError` or a helper function to reduce boilerplate. ✅ Fixed – added `into_internal_error()` helper and applied across all handlers.

17. **`notification.rs` – Inconsistent auth guard pattern**
    Some endpoints use `match identity { Some/None }` while others use `if identity.is_some()`. Pick one pattern for consistency. Consider an extractor or middleware for auth-required endpoints.

18. **Integration tests in `main.rs` require a live database**
    Tests (`test_get_regattas`, `test_get_heats`) connect to a real DB, making them integration tests that can't run in CI without infrastructure. Consider separating unit and integration tests.

### Minor / Cosmetic

19. **`config.rs:15` – Stale doc comment** mentions `Config::get()` which doesn't exist.

22. **`build.rs`** not reviewed but uses `built` crate — standard pattern, no concerns.

## Summary
The module is well-organized with good separation of concerns. The main areas for improvement are: fixing the `INFOPORTAL_V2` string bug, removing dead code (wrap_fn), addressing the inconsistent admin username checks, avoiding blocking calls in async contexts (monitoring sleep, std::sync::mpsc), and reducing boilerplate in error handling. Security-wise, the session key generation and plain-text DB password storage should be addressed for production use.