# Developer Commands

```bash
# Rust backend (workspace with 4 crates)
cargo build
cargo test        # Requires DB_HOST, DB_NAME, DB_USER, DB_PASSWORD env vars
cargo clippy     # Linting
cargo fmt       # Formatting

# UI5 frontend (SAP Fiori)
cd static && npm install
cd static && npm run build      # Builds to static/dist
cd static && npm run ts-typecheck
```

# Architecture

- **Monorepo**: Rust workspace with crates `aquarius`, `db`, `infoportal`, `timekeeper`
- **Frontend**: UI5 SAP Fiori app in `static/`
- **Entry points**: Each crate has its own `main.rs`; `infoportal` is the main web app
- **Database**: MS-SQL via Tiberius, uses env vars from `.env`
- **Rust edition**: 2024, toolchain 1.95.0 (requires nightly or very recent stable)

# CI / Build

- Runs on push/PR to main: `cargo fmt --check`, `cargo clippy`, `cargo test`
- Test requires database connection (env vars injected in CI)

# Key Files

- `Cargo.toml` - workspace root
- `static/package.json` - UI5 frontend
- `.env` - local env config