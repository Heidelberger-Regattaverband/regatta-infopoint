---
name: review
description: Review this regatta-infopoint repository, individual modules, or proposed changes for correctness, security, design, maintainability, and test coverage. Write prioritized, evidence-backed findings to review.md reports.
---

# Review

Review the Rust backend and SAP OpenUI5 TypeScript frontend for actionable issues, not stylistic preferences. Paths and commands are relative to the repository root.

## Scope and safety

- Read applicable `AGENTS.md` files, `git status --short`, and existing reports. Preserve unrelated changes and report history; verify documented configuration against manifests, `rust-toolchain.toml`, and CI.
- Follow the requested scope. Default repository scope: `aquarius/`, `db/`, `infoportal/`, `timekeeper/`, `ui5/`, and shared build/deployment configuration. For changes, inspect the requested diff and affected callers, separating regressions from pre-existing issues; ask if the base is ambiguous.
- Review source, not generated `ui5/dist/`, except for source/build consistency. Write reports only; do not fix code, upgrade dependencies, regenerate assets, or change configuration unless asked.
- Never expose credentials, keys, auth headers, cookies, or personal data, including secrets in `.env` and TLS files.
- Inspect integration-test setup and side effects first. Database-writing tests and production access require explicit authorization; use an approved disposable test database.

## Workflow

1. Trace Aquarius TCP events → database/cache → REST/WebSocket → UI5, and timekeeper input → persisted timestamps. Read implementations, callers, configuration, and tests.
2. Prioritize critical bugs, security, design, then minor improvements. Start with authentication, writes, timing/results, and failure recovery.
3. Check empty/missing/null data, malformed input, duplicates, disconnects, concurrency, and stale data.
4. Validate findings through concrete code paths, reproductions, tests, or safe targeted checks. Treat unresolved assumptions as questions; consolidate findings with one root cause.
5. Run feasible validation, distinguishing environment blockers from defects. Write scoped reports and an overall summary with coverage gaps, not an unsupported clean bill of health.

## Project-specific checks

### Rust / async
- Require idiomatic Rust that follows Rust best practices, especially ownership/borrowing, type-safe modeling, trait/API design, and error handling. Flag concrete deviations, not personal style preferences.
- Check `Result`/`Option`, error context, ownership, cancellation/shutdown, and workspace unsafe-code/clippy rules. Flag reachable panics, unchecked indexing, and failure-prone `unwrap`/`expect`, not every occurrence.
- Under live-regatta load, check blocking async work, locks across `.await`, unbounded tasks/channels, timeouts, backpressure, and error propagation.

### `db/`
- Check tiberius parameter bindings, dynamic identifier validation, joins/filters/order, nulls/conversions, and empty/not-found behavior. Verify regatta/race/heat/club/athlete scoping, scoring ties/status, and timestamp precision/order. Do not invent domain rules or assume ownership of the external Aquarius schema.
- Check per-user write pools, affected rows, multi-step consistency, pool exhaustion, cache keys/TTLs/invalidation, concurrency, and user isolation.

### `aquarius/`
- Check framing, partial reads, malformed messages, EOF, and size limits; parsing must not crash or desynchronize the stream.
- Check reconnect/backoff, timeouts, ordering, duplicates, shutdown, and how lost events affect caches and clients.

### `infoportal/`
- Verify per-handler authentication, object-level authorization, and per-user pools, especially writes/WebSockets; do not assume middleware. Check cookies, login/logout, CSRF for cookie-authenticated writes, TLS, and deployment-aware rate limits.
- Check validation, payload limits, safe errors/status codes (`DbError::NotFound` → 404), and agreement with OpenAPI and `doc/rest_api.http`. Review WebSocket lifecycle/heartbeats/backpressure and exposure through monitoring, logs, metrics, or static files.

### `timekeeper/`
- Check timestamp precision/order, start/finish association, duplicates, and race/heat/bib assignment.
- Ensure write failures reach the operator, retries avoid duplicates, and cancellation/terminal shutdown preserves accepted input.

### `ui5/`
- Use UI5 MCP for API/lifecycle questions; use the Web Components server only for Web Components. Check controllers/views/fragments, bindings, routes, types, async errors, and late responses overwriting newer navigation/filter state.
- Check listener/timer/WebSocket/fragment/Leaflet cleanup, reconnects, loading/error/empty states, and escaping (including notifications/map popups). Hidden controls are not authorization.
- Check accessibility, keyboard use, stable IDs, German/English i18n, and build/source consistency. Do not hand-edit generated bundles or report personal UI preferences.

### Shared build / deployment / tests
- Verify agreement among toolchain/manifests, Docker stages, frontend scripts, and CI. Check dependency compatibility; cite sources for vulnerability claims.
- Check least privilege, secrets, health checks, and production defaults; explain concrete risks rather than generic hardening.
- Tie regression-test gaps to specific failure paths, authorization, concurrency, reconnects, or timing behavior.

## Validation

Use the pinned toolchain and existing dependencies/scripts; do not override versions or install/upgrade dependencies to make checks pass.

| Area | Checks | Notes |
| --- | --- | --- |
| Rust formatting | `cargo fmt --check` | Non-mutating. |
| Rust linting | `cargo clippy -- -D warnings` | Match CI; scope crates as appropriate. |
| Rust tests | `cargo test --workspace` | Integration tests need DB env vars and safety checks above; prefer safe targeted tests if unavailable. |
| Frontend types/lint | `npm run ts-typecheck`, `npm run lint` in `ui5/` | Existing dependencies required; lint uses ui5lint. |
| Frontend build | `npm run build` in `ui5/` | Optional; requires approval because it rewrites `ui5/dist/`. |

Report actual results, failures, skipped checks, and blockers (toolchain, dependencies, DB access) separately from code findings. Never claim an unrun check passed.

## Severity and evidence

- **Critical:** demonstrated severe data loss/corruption, broad unauthorized access, or realistic complete service failure.
- **High:** significant security exposure, incorrect results/timing, or reachable core-feature failure.
- **Medium:** bounded correctness, reliability, performance, or maintainability impact.
- **Low:** material but limited readability, consistency, or usability improvement; no cosmetic nitpicks.

Rate impact and likelihood, not fix effort. Separate speculative risks and architectural suggestions from confirmed findings.

## Reports

- Write module findings to `{aquarius,db,infoportal,timekeeper,ui5}/review.md`; use root `review.md` for shared/cross-module findings and the overall summary. Link related locations rather than duplicate findings.
- Preserve existing history; append a dated, scoped section by default. Mark earlier issues resolved only after verifying current code.
- Record date, scope/base revision or working-tree state, reviewed areas, checks/results, and limitations. Order findings by severity and use current repository-relative paths/lines.

Use this finding format:

```markdown
### [High] Short, actionable title
- Location: `path/to/file.rs:42-48`
- Description: Trigger, failure, and impact.
- Evidence: Code path or reproduction; distinguish observation from inference.
- Suggested fix: Smallest practical correction consistent with surrounding design.
- Regression test: Scenario and expected behavior.
```

For areas without actionable findings, say so and list checks/coverage. End with key findings, report paths, validation results, and blockers; no unsupported approval verdict.
