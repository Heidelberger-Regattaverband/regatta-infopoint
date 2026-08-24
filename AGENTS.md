# Developer Commands

```bash
# Rust backend
cargo build
cargo build --release
cargo test --workspace     # Requires DB_HOST, DB_NAME, DB_USER, DB_PASSWORD env vars
cargo clippy -- -D warnings
cargo fmt --check

# UI5 frontend (SAP OpenUI5 TypeScript)
cd static && npm install
cd static && npm run build          # Builds TypeScript + assets to static/dist/
cd static && npm run build:opt      # Self-contained optimized build
cd static && npm run ts-typecheck   # tsc --noEmit
cd static && npm run lint           # ui5lint
cd static && npm run watch          # Watch mode for development
```

# Architecture

**Regatta information system** for the Heidelberger Regatta-Verband. Reads from a Microsoft SQL Server database managed by the Aquarius regatta management software and serves live race data, schedules, scoring, athlete/club info, and timekeeping via a web UI.

## Workspace Structure

```
regatta-infopoint/
  Cargo.toml        # workspace root (resolver = "3", edition = "2024", rust-version = "1.98.0")
  aquarius/         # Aquarius TCP client library (real-time event streaming from Aquarius)
  db/               # MS-SQL database layer: models, queries, connection pool, cache (library)
  infoportal/       # Main web server / REST API (binary — the deployed service)
  timekeeper/       # Terminal UI tool for live race timekeeping (binary)
  static/           # SAP OpenUI5 TypeScript frontend (webapp/ → dist/)
  doc/              # REST API HTTP file (rest_api.http)
  .github/          # CI workflows + Dependabot
  Dockerfile        # Multi-stage build (rust:1.98.0 builder, ubuntu:26.04 runtime)
  .env              # Local dev environment variables
```

## Crates

### `db` (library)
Core database access layer. All domain models, raw SQL queries (tiberius, no ORM), bb8 connection pool (max 80 / min 30 idle), and stretto in-memory cache (TinyLFU, TTL-based, default 60s).

Key modules:
- `db/src/aquarius.rs` — `Aquarius` struct: high-level query interface with cache-aside (`compute_if_missing`)
- `db/src/aquarius/model/` — 17 domain model structs (`Regatta`, `Race`, `Heat`, `HeatEntry`, `HeatResult`, `Entry`, `Athlete`, `Crew`, `Club`, `AgeClass`, `BoatClass`, `Block`, `Schedule`, `Score`, `Statistics`, `Notification`, `Referee`), each with `From<&Row>` impl using named column constants
- `db/src/tiberius/` — `connection.rs`, `pool.rs` (global pool), `user_pool.rs` (per-user pool for admin writes), `row_column.rs` (typed row deserialization traits)
- `db/src/cache.rs` — `Cache<K,V>` / `Caches` abstraction
- `db/src/timekeeper/` — `Timestamp` (persist start/finish times), `TimeStrip` (ordered timestamps)
- `db/src/aquarius/flags_scraper.rs` — Scrapes athlete flag images

### `aquarius` (library)
TCP client for the Aquarius native protocol. Provides real-time event streaming (`event.rs`, `messages.rs`, `client.rs`). Consumed by `infoportal` to drive WebSocket push on heat state changes.

### `infoportal` (binary — main web server)
Actix-Web server. REST API + WebSocket endpoints, static file serving (the UI5 SPA), Swagger UI at `/swagger-ui/`, Prometheus metrics at `/metrics`.

Key modules:
- `infoportal/src/config.rs` — `Config` singleton via `LazyLock`, all env var config with defaults
- `infoportal/src/auth.rs` — HTTP Basic Auth extraction, per-user pool for write operations
- `infoportal/src/http/server.rs` — HTTP+HTTPS listeners, rustls TLS, rate limiting, actix-session + actix-identity, Prometheus
- `infoportal/src/http/rest_api/` — Handler modules: `athlete`, `authentication`, `club`, `misc` (scoring/statistics/schedule), `monitoring` (health/metrics WS), `notification`, `race`, `timekeeping` (WS)
- `infoportal/src/http/api_doc.rs` — utoipa OpenAPI aggregation
- `infoportal/build.rs` — embeds git hash + build timestamp via `built` crate

Authentication: `Option<Identity>` on read endpoints; `auth::authenticate` → per-user pool for writes. No middleware — per-handler checks.

### `timekeeper` (binary)
Standalone ratatui TUI for entering race start/finish timestamps at the finish line. Uses the same `db` library. CLI args via clap. Tabs: heats, timestrip, logs.

## Frontend (`static/`)
SAP OpenUI5 TypeScript SPA. 20+ XML views (Launchpad, RacesTable, HeatsTable, HeatDetails, ClubsTable, AthleteDetails, ScoringTable, ScheduleTable, Statistics, Timekeeping, Map, Monitoring, Admin, etc.). One TypeScript controller per view. Leaflet 1.9.x for the map view. i18n: German + English.

Build: `@ui5/cli` v4 (ui5.yaml, specVersion 4.0) with `ui5-tooling-modules-task` (npm bundle) and `ui5-tooling-transpile-task` (TS → JS). Built assets committed to `static/dist/` and served from Docker.

## Database
MS-SQL Server (Aquarius schema — externally managed, no migrations in this repo). Raw parameterized SQL via tiberius (`@P1`, `@P2`, ...). Column names as `const` strings to reduce typos. Authentication uses SQL Server users; admin ops get a dedicated per-user connection pool.

# REST API

Base URL: `http://localhost:8080` (local), `https://info.regatta-hd.de` (production). Full API reference: `doc/rest_api.http`.

**Public endpoints:**
- `GET /api/active_regatta`
- `GET /api/regattas/{id}/races` · `/heats` · `/clubs` · `/athletes` · `/filters` · `/notifications` · `/scoring` · `/statistics` · `/schedule`
- `GET /api/heats/{id}` · `/races/{id}` · `/athletes/{id}` · `/athletes/{id}/entries`
- `GET /api/regattas/{id}/clubs/{club_id}` · `/clubs/{club_id}/entries`
- `GET /api/regattas/{id}/races/club-conflicts`
- WS `GET /api/monitoring` · `/api/timekeeping`

**Authenticated (HTTP Basic / session):**
- `POST /api/login` · `POST /api/logout` · `GET /api/identity`
- `POST /api/regattas/{id}/notifications` · `PUT /api/notifications/{id}` · `DELETE /api/notifications/{id}` · `POST /api/notifications/{id}/read`

# Configuration (Environment Variables)

| Variable | Default | Required | Description |
|---|---|---|---|
| `DB_HOST` | — | YES | MS-SQL server hostname |
| `DB_PORT` | `1433` | — | MS-SQL port |
| `DB_NAME` | — | YES | Database name |
| `DB_USER` | — | YES | DB username |
| `DB_PASSWORD` | — | YES | DB password |
| `DB_ENCRYPTION` | `false` | — | Enable TLS for DB connection |
| `DB_POOL_MAX_SIZE` | `80` | — | Max pool connections |
| `DB_POOL_MIN_IDLE` | `30` | — | Min idle connections |
| `HTTP_PORT` | `8080` | — | HTTP port |
| `HTTPS_PORT` | `8443` | — | HTTPS port |
| `HTTPS_CERT_PATH` | `./ssl/cert.pem` | — | TLS certificate |
| `HTTPS_KEY_PATH` | `./ssl/key.pem` | — | TLS private key |
| `HTTP_RL_MAX_REQUESTS` | `500` | — | Rate limit requests |
| `HTTP_RL_INTERVAL` | `600` | — | Rate limit window (seconds) |
| `HTTP_APP_CONTENT_PATH` | `./static/dist` | — | Static files path |
| `ACTIVE_REGATTA_ID` | (auto from DB) | — | Override active regatta |
| `CACHE_TTL` | `60` | — | Cache TTL in seconds (max 3600) |
| `AQUARIUS_HOST` | `aquarius` | — | Aquarius TCP host |
| `AQUARIUS_PORT` | `2048` | — | Aquarius TCP port |
| `RUST_LOG` | — | — | Tracing filter |

Local `.env` example:
```
DB_HOST=data
DB_NAME=Regatta_2026
DB_USER=info
DB_PASSWORD=portal
RUST_LOG=infoportal=info
```

# CI / Build

Three GitHub Actions workflows:

- **`ci.yaml`** — push/PR to main: `cargo fmt --check` → `cargo clippy -- -D warnings` → `cargo test --workspace` (with Tailscale VPN + DB secrets for integration tests)
- **`build_and_push_docker.yml`** — push to main: builds and pushes `ofterdinger/regatta-infoportal:latest` (linux/amd64) to Docker Hub
- **`coverage.yml`** — push to main: `cargo llvm-cov --workspace --all-features --codecov`, uploads to Codecov

Docker image: multi-stage (`rust:1.98.0` builder with Node.js 24, `ubuntu:26.04` runtime, non-root UID 1001, ports 8080/8443).

# Rust Toolchain

- Pinned: **1.98.0** (stable), edition **2024**
- `unsafe_code = "forbid"` and `unsafe_op_in_unsafe_fn = "forbid"` enforced workspace-wide
- All clippy lints set to `warn`
- Release profile: `lto = "fat"`, `codegen-units = 1`

# Key Files

- `Cargo.toml` — workspace root (resolver 3, shared dependencies, workspace lints)
- `static/package.json` — UI5 frontend deps and scripts
- `static/ui5.yaml` — UI5 Tooling config (specVersion 4.0)
- `.env` — local dev config
- `doc/rest_api.http` — full REST API reference with example requests
- `ssl/cert.pem`, `ssl/key.pem` — local TLS certificates

# Code Review Guidelines

- Focus on correctness, security, design, and maintainability.
- For each issue provide: description, file + line number, suggested fix, severity (critical / high / medium / low).
- Avoid nitpicks unless they impact readability or consistency.
- Consider Rust best practices (idiomatic error handling, no unsafe, clippy compliance).
- Consider SAP OpenUI5 and TypeScript best practices for the `static/` module.
- Store review feedback in a `review.md` file in the corresponding crate.
- Prioritize: critical bugs > security issues > design flaws > minor improvements.

Known open issues (tracked in `db/review.md` and `infoportal/review.md`):
- TLS for DB connection disabled by default (`DB_ENCRYPTION=false`)
- Raw error messages can leak to API consumers
- `CacheQueryParams` struct duplicated across handlers
- Magic number `64` used for "final round" in multiple places
- No auth middleware — authentication is checked per-handler

# MCP Servers

- For UI5 related tasks or questions use MCP `@ui5/mcp-server`
- For UI5 Web Components use MCP `@ui5/webcomponents`
