---
name: review
description: Review this regatta-infopoint repository, individual modules, or proposed changes for correctness, security, design, maintainability, and test coverage. Write prioritized, evidence-backed findings to review.md reports.
---

# Review

Review the Rust backend and SAP OpenUI5 TypeScript frontend of the regatta information system. Focus on actionable issues, not stylistic preferences.

All project paths and commands below are relative to the repository root, not this skill's directory.

## Scope and safety

1. Read the root `AGENTS.md` and any applicable nested instructions. Confirm current configuration in manifests, `rust-toolchain.toml`, and CI workflows; documentation may be outdated.
2. Use the scope requested by the user. For an unspecified repository review, cover `aquarius/`, `db/`, `infoportal/`, `timekeeper/`, `static/`, and shared build/deployment configuration. For a change review, inspect the requested diff and affected callers; separate pre-existing issues from regressions.
3. Inspect `git status --short` and existing review reports before writing. Preserve unrelated changes and existing report history. Review source files rather than generated `static/dist/` bundles, except when checking source/build consistency.
4. Review only: do not fix application code, upgrade dependencies, regenerate assets, or change configuration unless asked. Writing the review reports is part of the review task.
5. Never print credentials, private keys, authorization headers, session cookies, or personal data. Do not copy secrets from `.env` or TLS files into reports.
6. Do not run database-writing tests or contact production services without explicit authorization. Check test setup and side effects before running integration tests; use an approved disposable test database.

## Workflow

1. Map the relevant data flow: Aquarius TCP events → database/cache → REST or WebSocket handlers → UI5 models/controllers, and timekeeper input → persisted timestamps.
2. Read implementations, callers, configuration, and relevant tests before reporting an issue. For changes, compare against the requested base; ask if the base is ambiguous rather than assuming a branch.
3. Prioritize critical bugs, security issues, design flaws, then minor improvements. Trace authentication boundaries, write operations, timing/results correctness, and failure recovery first.
4. Check error paths and boundary cases: empty results, missing records, null columns, malformed input, duplicate events, disconnects, concurrent requests, and stale data.
5. Validate each suspected issue with a concrete code path, reproduction, existing test, or safe targeted check. Mark unresolved assumptions as questions, not confirmed defects.
6. Run applicable checks from the validation section when feasible. Record failures and skipped checks accurately; distinguish environment failures from application defects.
7. Write module reports and a concise overall summary. State which areas were reviewed and any remaining coverage gaps; do not imply the repository is defect-free when no issues were found.

## Project-specific checks

### Rust and asynchronous behavior

- Use idiomatic `Result`/`Option` handling and meaningful error context. Flag reachable panics, unchecked indexing, and `unwrap`/`expect` in failure-prone paths, not every occurrence indiscriminately.
- Respect workspace unsafe-code prohibitions and clippy requirements. Check resource ownership, cancellation, task shutdown, and error propagation.
- Look for blocking work on async workers, locks held across `.await`, unbounded channels/tasks, and missing timeouts or backpressure. Consider load during a live regatta, not only single-request behavior.

### `db/`: SQL, cache, and domain correctness

- Parameterize values with tiberius bindings; validate any dynamic SQL identifiers. Check joins, filters, ordering, null handling, numeric conversions, and empty/not-found behavior against the externally managed Aquarius schema.
- Verify regatta, race, heat, club, and athlete IDs are scoped correctly. Check scoring ties, result status, and timestamp precision/ordering without inventing undocumented domain rules.
- Ensure writes use the appropriate per-user connection pool, check affected rows, and preserve consistency across multi-step changes. Do not propose schema migrations as if this repository owns the schema.
- Check cache keys, TTLs, invalidation after writes/events, concurrency, and user-specific data isolation. Ensure pool limits and error paths cannot exhaust connections.

### `aquarius/`: event streaming

- Check framing, partial reads, malformed messages, EOF handling, and input size limits. Ensure parsing errors cannot crash or silently desynchronize the stream.
- Review reconnect/backoff, timeout behavior, event ordering, duplicate handling, and shutdown. Trace how disconnected or dropped events affect cached data and connected clients.

### `infoportal/`: API and security

- Verify authentication and authorization in each handler, especially writes and WebSocket endpoints; do not assume middleware enforces them. Check object-level access and per-user pool separation.
- Review session cookie flags, login/logout behavior, CSRF protections for cookie-authenticated writes, TLS configuration, and rate limiting with the actual deployment context.
- Check request validation, payload limits, safe error responses, and correct HTTP status codes, including `DbError::NotFound` → 404. Compare handlers with OpenAPI definitions and `doc/rest_api.http`.
- Review WebSocket lifecycle, heartbeat/timeouts, slow-client backpressure, and sensitive information exposed through monitoring, logs, metrics, or static file serving.

### `timekeeper/`: persisted race timing

- Check timestamp precision, chronological ordering, start/finish association, duplicate input, and race/heat/bib assignment.
- Ensure failed writes are visible to the operator, retries do not duplicate data, and cancellation or terminal shutdown does not silently lose accepted input.

### `static/`: OpenUI5 and TypeScript

- Use the configured UI5 MCP server for API/lifecycle questions instead of guessing control behavior. Use the UI5 Web Components server only if Web Components are involved.
- Review controller/view/fragment consistency, model bindings, route parameters, TypeScript types, and asynchronous error handling. Check that late responses cannot overwrite newer navigation or filter state.
- Check cleanup of event listeners, timers, WebSockets, fragments, and Leaflet maps. Review loading/error/empty states and reconnect behavior.
- Check escaping of user-controlled content, including notification text and map popups. Never treat hidden UI controls as authorization.
- Review accessibility, keyboard interaction, stable IDs, and German/English i18n consistency. Flag only concrete usability or maintainability problems, not personal UI preferences.
- Check build configuration and whether deployed assets correspond to source changes; avoid hand-editing generated bundles.

### Shared configuration and deployment

- Verify Rust toolchain/manifests, Docker build/runtime stages, frontend build scripts, and CI agree. Review dependency changes for compatibility and relevant known vulnerabilities, citing sources when making vulnerability claims.
- Check least-privilege runtime/CI permissions, secret handling, health checks, and production-safe defaults. Explain the concrete risk rather than recommending generic hardening without context.
- Look for regression coverage of failure paths, authorization boundaries, concurrent updates, reconnects, and timing correctness. Tie missing-test suggestions to a specific behavior at risk.

## Validation

Use the pinned toolchain and existing scripts; do not silently override toolchain versions or install/upgrade dependencies to make checks pass.

| Area | Checks | Notes |
| --- | --- | --- |
| Rust formatting | `cargo fmt --check` | Non-mutating formatting check. |
| Rust linting | `cargo clippy -- -D warnings` | Match CI; scope to affected crates when appropriate. |
| Rust tests | `cargo test --workspace` | Requires database environment variables for integration tests. Inspect side effects and obtain authorization for writes first. Prefer safe targeted tests when full tests are unavailable. |
| Frontend types | `npm run ts-typecheck` in `static/` | Requires existing frontend dependencies. |
| Frontend linting | `npm run lint` in `static/` | Uses ui5lint. |
| Frontend build | `npm run build` in `static/` | Optional when needed; rewrites `static/dist/`, so obtain approval before rebuilding in a review-only task. |

Never claim a check passed unless it ran successfully. Report unavailable toolchains, dependencies, database access, and other blockers separately from code findings.

## Severity and evidence

- **Critical:** demonstrated severe data corruption/loss, broad unauthorized access, or complete service failure under realistic conditions.
- **High:** significant security exposure, incorrect race results/timing, or failure of a core feature with a reachable trigger.
- **Medium:** bounded correctness, reliability, performance, or maintainability issue with a concrete impact.
- **Low:** limited-impact improvement that materially affects readability, consistency, or usability. Omit cosmetic nitpicks.

Base severity on impact and likelihood, not how easy the fix is. Keep speculative risks and architectural suggestions separate from confirmed findings. Consolidate duplicate manifestations of one root cause.

## Reports

- Store findings in the corresponding crate's `review.md`: `aquarius/review.md`, `db/review.md`, `infoportal/review.md`, or `timekeeper/review.md`. Use `static/review.md` for frontend findings and root `review.md` for shared configuration or cross-module findings and the overall summary.
- Read existing reports before updating them. Preserve unrelated entries/history; append a dated, scoped review section by default. Only mark an earlier issue resolved after verifying the current code.
- For each report, record date, scope/base revision or working-tree state, reviewed areas, checks run and their results, and limitations. Order findings by severity.
- For each issue include a concise description, current repository-relative file path and line number/range, trigger and impact, supporting evidence, suggested fix, and a regression-test suggestion. Link related locations for cross-module issues instead of duplicating the finding.

Use this finding format:

```markdown
### [High] Short, actionable title
- Location: `path/to/file.rs:42-48`
- Description: What fails, under which conditions, and why it matters.
- Evidence: Relevant code path or reproduction; distinguish observation from inference.
- Suggested fix: Smallest practical correction consistent with the surrounding design.
- Regression test: Scenario and expected behavior that would prevent recurrence.
```

If no actionable findings exist in an area, say so and list what was checked. End the user-facing response with the most important findings, report paths, validation results, and blockers. Do not add an unsupported approval verdict.
