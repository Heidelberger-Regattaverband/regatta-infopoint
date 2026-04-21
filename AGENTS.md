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

# Code Review Guidelines
- Focus on correctness, security, design, and maintainability.
- For each issue, provide:
  - A clear description of the problem
  - The file and line number(s) where it occurs
  - A suggested fix or improvement
- Avoid nitpicks unless they impact readability or consistency.
- Consider the overall architecture and how components interact.
- Look for patterns of code duplication or inconsistency.
- Consider Rust best practices and idiomatic usage.
- Store the review feedback in a `review.md` file in the corresponding crate for reference and tracking.
- Prioritize issues based on severity: critical bugs > security issues > design flaws > minor improvements.