# Code Review: `static/` UI5 Frontend

**Date:** 2026-05-18  
**Scope:** All source files in `static/` (SAPUI5 / TypeScript SPA)  
**Tooling:** TypeScript `^6.0.3` (strict), `@openui5/types` `^1.148.0`, `@ui5/cli` `^4`, `@ui5/linter` `^1.21`

---

## Summary

The UI is a SAPUI5 SPA implemented in TypeScript, organised along the standard MVC pattern with one controller per view, a small set of fragments, an i18n resource bundle (`de` / `en`), JSON-model based state, and a static `Formatter`. Routing and targets are fully declarative in `manifest.json`, TypeScript is configured `strict: true`, and the `Component` lifecycle uses a sensible memoised-promise bootstrap.

The codebase is in good overall shape but suffers from a few structural and correctness issues worth tackling: ad-hoc REST/`$.ajax` calls scattered across controllers, heavy use of `any` (which silently undermines the strict TS configuration), incomplete typing of model objects, no automated tests, and several inconsistencies between the standardised `BaseController` helpers and the actual controllers. There are also a handful of small correctness / UX bugs.

---

## Issues

### 1. Inconsistent HTTP layer: `$.ajax` mixed with `JSONModel.loadData` — **Design / Maintainability** (medium)

- **Files:** `controller/Launchpad.controller.ts` (lines 72–80, 130–172), `controller/Admin.controller.ts` (lines 133–243), vs. all other controllers using `super.updateJSONModel(...)`.
- **Problem:** `Launchpad` and `Admin` use jQuery `$.ajax` directly for REST calls (login, logout, identity, notifications CRUD). All other controllers go through `BaseController.updateJSONModel` → `JSONModel.loadData`. This:
  - introduces two parallel error-handling styles (one toasts, one logs to `console.error`, one shows `MessageToast`),
  - couples the UI to jQuery although UI5 does not require it,
  - makes it impossible to centralise concerns such as CSRF tokens, auth headers, generic 401 handling, or rate limiting.
- **Suggested fix:** Introduce a small typed REST service (e.g. `model/ApiService.ts`) using `fetch`/`JSONModel.loadData`, with methods like `login(creds)`, `getIdentity()`, `createNotification(...)`. Have all controllers go through it, drop the jQuery dependency from controllers, and centralise the toast styling logic (the `.sapMMessageToast<X>` class hack — see also issue 6).

### 2. Authentication flow blends transport and view state — **Security / Design** (high) — ✅ Fixed (2026-05-19)

- **File:** `controller/Launchpad.controller.ts`, lines 126–179
- **Problem:**
  1. `login()` posts credentials via `$.ajax`, but the response is trusted as `{ username, scope }` and written into the `identity` model. `performLogin()` does not return a Promise, so callers cannot `await` the result and any code immediately after `this.login()` sees stale state.
  2. The `getIdentity()` call on `onInit` runs on every Launchpad show; if the server responds with 401 the controller silently sets `authenticated: false`. The path is duplicated with the login error handler and not unified with server-driven session errors (e.g., the next `JSONModel.loadData` in another controller will also 401, with no global handler).
  3. The `password` field is reset locally after submit, but credentials live in `this.credentialsModel` registered as a *view* model on the Launchpad view; ensure that model is destroyed/cleared when the popover closes. There is no explicit binding-mode constraint on the credentials model — `JSONModel`'s default is two-way, which is fine for the input but means the password lingers in memory.
- **Suggested fix:** Make `performLogin()` `async` and return a Promise; update `identity` only on success. Reset *both* fields of the credentials model immediately after submission and on popover close. Add a single global 401 handler (e.g., subscribe to UI5’s `requestFailed` or wrap `fetch`) so the app reverts to the unauthenticated UI consistently across views.
- **Resolution:**
  1. `performLogin()` and `login()` now return `Promise<void>`; the identity model is only mutated inside the success branch (`Launchpad.controller.ts` `login()`), and the popover is closed before the request so callers see consistent state.
  2. Both username **and** password fields are wiped via the new `clearCredentials()` helper after every login attempt (success *or* failure) and again on the popover's `afterClose` event (`onLoginPopoverAfterClose`, wired in `view/LoginPopover.fragment.xml`).
  3. A global 401 handler now lives in `Component.attachUnauthorizedHandler` / `handleUnauthorized`. It is attached to all component-owned `JSONModel`s (regatta, filters, notifications, race, heat) so any `loadData` returning `401 Unauthorized` reverts the identity model to the anonymous state — unifying session-expiry handling across views without each controller re-implementing the check.

### 3. `noImplicitAny` is configured, yet `any` is used pervasively — **Maintainability** (medium)

- **Files:** Almost every controller; most prominent in `BaseTable.controller.ts` (lines 70–95, 200–211), `RaceDetails.controller.ts` (lines 52, 88, 100, 110), `HeatDetails.controller.ts`, `RacesTable.controller.ts`, `Admin.controller.ts`, `Timekeeping.controller.ts`.
- **Problem:** Many fetched JSON shapes are typed as `any` (`const heat: any = ...`, `const entry: any = ...`, `const data: any = JSON.parse(...)`). This bypasses the strict TS configuration — refactoring a backend field becomes invisible to the compiler and IDE auto-completion is degraded.
- **Suggested fix:** Extend `model/types.ts` with the runtime DTO shapes the backend returns (Race, Heat, Entry, Athlete, Club already exist — but they are partial and not used everywhere). Use them in the controllers via `getProperty<Race>(path)` (or a small typed helper around `Context.getObject<T>()`). Add an opt-in `_nav?: { isFirst?: boolean; isLast?: boolean; back?: string; disabled?: boolean }` member to a shared base interface so that `any` is not needed at all.

### 4. `_nav` metadata is mutated on bound JSON model objects — **Design** (medium)

- **Files:** `BaseTable.controller.ts:208`, `RacesTable.controller.ts:80`, `HeatsTable.controller.ts:108`, `AthleteDetails.controller.ts:46`, `ClubDetails.controller.ts:50`, `RaceDetails.controller.ts:92`, `HeatDetails.controller.ts:48–53`.
- **Problem:** Several controllers mutate `item._nav = { ... }` on objects fetched from the backend. UI5’s `JSONModel` does not detect such direct mutations; while it works because the model is later re-rendered or the data is read out by hand, it is implicit coupling and easy to break. It also leaks UI navigation state into objects that conceptually represent backend data and survives across navigations as long as the `race`/`heat` Component-models are alive.
- **Suggested fix:** Keep navigation state in a dedicated view-side model (e.g., a `nav` `JSONModel` with `{ isFirst, isLast, back, disabled }`) and bind the detail-view’s nav buttons to it. The selection logic should set the state on the nav model, not on the data object. This also makes the bug in `RaceDetails.onNavBack` (which reads `_nav` from the *previous* race) impossible.

### 5. No central API service → URL paths duplicated across controllers — **Code Duplication** (medium)

- **Files:** `RacesTable.controller.ts:144`, `HeatsTable.controller.ts:172`, `ClubsTable.controller.ts:82`, `AthletesTable.controller.ts:85`, `ClubDetails.controller.ts:103–104`, `AthleteDetails.controller.ts:69–70`, `RaceDetails.controller.ts:105`, `HeatDetails.controller.ts:85`, `ScheduleTable.controller.ts:60`, `ScoringTable.controller.ts:60`, `Statistics.controller.ts:41`, `Map.controller.ts:62`, `Admin.controller.ts:131, 158, 184, 209, 232`, `Component.ts:202, 216, 224`.
- **Problem:** Every controller hand-builds REST URLs of the form `` `/api/regattas/${regatta.id}/...` ``. There is no helper, so the path scheme is implicit and a backend change has to be reflected in many files. Some paths (e.g. `calculateScoring`) are camel-case while most are lower-case — a central helper would expose this inconsistency.
- **Suggested fix:** Introduce `model/Endpoints.ts` (or methods on the `ApiService` from issue 1) like `endpoints.regatta.races(regattaId)`, `endpoints.athletes.details(regattaId, athleteId)`. Single grep target for backend contract changes.

### 6. Toast styling via `$(".sapMMessageToast").addClass(...)` is fragile — **Correctness** (medium)

- **Files:** `Base.controller.ts:36–47`, `Launchpad.controller.ts:138,142`, `Admin.controller.ts:109,142,168,175,201,216,222,235,241`.
- **Problem:**
  1. `MessageToast.show(...)` is asynchronous; the toast DOM element is created right after `show()` returns, so the immediate `$(".sapMMessageToast")` selector frequently runs *before* the element exists or matches a **previous** toast still fading out. The styling is therefore flaky.
  2. If multiple toasts are shown in quick succession, every `.sapMMessageToast` element gets every class added — color leaks across toasts.
  3. The CSS in `webapp/css/style.css:18–51` overrides with `!important`, indicating a previous fight with theme styles.
- **Suggested fix:** Use UI5’s built-in `sap.m.MessageStrip`, `MessageBox`, or `MessageToast` with the `styleClass` option (added in 1.115+). For one-off info/error feedback, `MessageBox.error/information` already carries the right semantic styling. At a minimum, scope the class application to only the newly-created element (e.g., via a UI5 `EventDelegate` on toast open) — but the cleanest fix is to drop the class hack and use semantic UI5 messages.

### 7. `BaseController.updateJSONModel` swallows errors — **Correctness / UX** (medium)

- **File:** `controller/Base.controller.ts`, lines 129–141
- **Problem:** The catch block logs `params.statusCode + ": " + params.statusText` and returns `false`. This is fine for the "table refresh failed" toast, but:
  - the log message is built without checking that `error` actually has the `Model$RequestFailedEventParameters` shape (fetch errors, network errors, type errors all become `"undefined: undefined"`),
  - the calling code aggregates with `succeeded[0] && succeeded[1]` (e.g. `AthleteDetails.controller.ts:57`), so a partial 200/500 combination yields a single generic toast and discards the failing URL,
  - the parameter is typed `error: any` — inconsistent with the rest of the controller code.
- **Suggested fix:** Type the parameter as `unknown`, log via `Log.error(...)`, distinguish between network/transport errors and HTTP errors, and return enough information for callers to react (e.g. 401 → re-login, 503 → retry hint).

### 8. `BaseTable.controller.ts` is doing too much — **Design / Single-Responsibility** (medium)

- **File:** `controller/BaseTable.controller.ts`, all
- **Problem:** The base table controller mixes:
  - event-bus subscription/unsubscription,
  - table growing logic (`growTable`),
  - filter dialog handling (`onHandleFilterDialogConfirm`, `createFilter`, `updateFilterBar`),
  - sort dialog handling (`onSortDialogConfirm`, `sortTable`),
  - search-filter merge with persistent filters (`setSearchFilters`, `applyFilters`),
  - selection navigation (first/previous/next/last),
  - and abstract `onItemChanged`.

  Several methods take untyped `parametersMap: any`; the encoded "filter mini-language" in `createFilter` (`split("___")`) reinvents UI5’s built-in filter-from-bindingpath mechanism and is a custom string protocol with no schema validation. A typo in a `ViewSettingsItem` `key` like `dateTime__Contains___X` (one underscore short) would be silently parsed into garbage filters.
- **Suggested fix:** Split the controller into focused mixins/helpers:
  - `TableNavigationMixin` (event bus + first/prev/next/last),
  - `TableFilterController` (filter dialog plumbing),
  - `TableSortController` (sort dialog plumbing).

  Replace the `___`-encoded filter keys with a small typed object passed via `customData` (e.g., `<core:CustomData key="filter" value="{path:'race/lightweight', op:'EQ', value:true}"/>` parsed with `JSON.parse`), or generate the `Filter` programmatically from a typed configuration array in the controller.

### 9. `getViewSettingsDialog` returns a non-null reference even before the fragment is loaded — **Correctness** (low)

- **File:** `controller/BaseTable.controller.ts`, lines 104–114
- **Problem:** The first line uses the non-null assertion: `let dialog: ViewSettingsDialog = this.viewSettingsDialogs.get(dialogFragmentName)!;`. Subsequent code immediately checks `if (!dialog)`, so the assertion is misleading. Worse, if `Fragment.load` rejects, the `Promise<ViewSettingsDialog>` returned from this method also rejects but two concurrent callers would both go through the load path because nothing memoises the in-flight promise (each call sees `undefined` until the first one resolves). The `Component`-level memoisation pattern in `Component.ts` is a good template — the same pattern is missing here.
- **Suggested fix:** Drop the non-null assertion (use `let dialog: ViewSettingsDialog | undefined =`), and memoise an in-flight `Promise<ViewSettingsDialog>` per fragment name like the `Component` does for the regatta and filter models, so the second caller awaits the same load.

### 10. Two HTML bootstrap files (`index.html` / `index_v2.html`) without documented purpose — **Maintainability** (low)

- **Files:** `webapp/index.html`, `webapp/index_v2.html`
- **Problem:** Both files exist next to each other; the only meaningful difference is the UI5 bootstrap URL (`https://sdk.openui5.org/1.148.0/...` vs. `https://sdk.openui5.org/nightly/2/...`). There is no comment explaining which is authoritative or under which circumstances `index_v2.html` is used. `manifest.json:start_url` points at `index.html`, so `index_v2.html` looks like dead code. Both also pin against `data-sap-ui-compat-version="edge"` and embed `data-sap-ui-async="true"`, which is good, but the duplication invites drift.
- **Suggested fix:** Either delete `index_v2.html` if unused, or add a top-of-file comment explaining its purpose (e.g. "preview / canary build against UI5 v2 nightly"). Document the relationship in `static/README.md` (if added — there is currently none for the UI module).

### 11. `WebSocket` JSON parsed and dispatched without schema — **Correctness / Robustness** (medium)

- **Files:** `Monitoring.controller.ts:158`, `Timekeeping.controller.ts:163–183`
- **Problem:** Each WebSocket message handler does `JSON.parse(event.data) as any` and then `if (data.AquariusHeats) ...`. There is no validation, no type guard, no central message dispatcher. If the backend emits a payload that *looks* similar (`data.aquariusHeats`, lowercase) the message is silently logged as "unknown event" and lost. Additionally, the messages mix command-style (`Update`, `AquariusHeats`, `Timestamp`, `TimeStrip`, `HeatsReadyToStart`, `error`) keys without a discriminator field, which is more error-prone than a `{ "type": "...", "payload": ... }` envelope.
- **Suggested fix:** Define a discriminated union in `model/types.ts`:
  ```ts
  type WsEvent =
    | { kind: "Timestamp"; payload: TimeStripEntry }
    | { kind: "TimeStrip"; payload: TimeStripEntry[] }
    | { kind: "Error"; message: string };
  ```
  Add a small `assertWsEvent(unknown): WsEvent` (or a dependency on a runtime-validation library like Zod) and route via a single `switch (data.kind)`. This also makes the protocol explicit and shareable with the Rust backend.

### 12. `MapController.connect` does not happen — but `Map`, `Monitoring` and `Timekeeping` lack reconnect logic — **Robustness** (medium)

- **Files:** `Monitoring.controller.ts:136–164`, `Timekeeping.controller.ts:145–184`
- **Problem:** Both WebSocket-using controllers create a `WebSocket` and assume the connection is permanent for as long as the route stays active. There is no automatic reconnect on `onclose` (e.g., when the laptop wakes up, switches Wi-Fi, or the server restarts). The user has to hit the status button manually. There is also no exponential back-off or message buffering.
- **Suggested fix:** Implement a small `ReconnectingSocket` wrapper that:
  - retries with exponential back-off + jitter on close (capped),
  - exposes a `RxJS`-style observable or simple typed event emitter,
  - flushes a small outbound queue once reconnected.

  This is especially important for the timekeeping screen where a missed *finish* timestamp is a real correctness issue.

### 13. `Map.controller` reads model data synchronously inside `getClubsLayerGroup` — **Correctness** (low)

- **File:** `controller/Map.controller.ts`, lines 28–35, 78, 123–139
- **Problem:** `loadMap()` is invoked twice from the `onInit` matched handler: once after `loadModel()` resolves, and once inside the `.catch` (defensive fallback). Inside `loadMap()`, `getClubsLayerGroup` reads `this.participatingClubsModel.getData()`. In the `.catch` path the model is never populated, so the clubs layer is built with zero markers — but `clubBounds` is then `latLngBounds([])` which is **invalid** (`isValid()` returns `false`), so `centerMap(true)` falls through to `regattaBounds`. Behaviour is correct but the comment at line 27 ("loadMap() is a no-op on subsequent calls because the map is created only once") is incorrect — `loadMap()` is no-op only when invoked twice in success, not in the error fallback.
- **Suggested fix:** Move the `loadMap()` call out of the `.catch` and into a `finally`, or simply call `loadMap()` *unconditionally* after the model resolves/rejects, and rely on the `if (!this.map)` guard. Update the comment to reflect actual behaviour.

### 14. `Component.bootstrap` overwrites `notifications` model with empty model on failure — **Correctness** (low)

- **File:** `Component.ts`, lines 130–137
- **Problem:** If `loadNotifications()` fails, the `Component` registers an empty `notificationsModel` so views can bind without runtime errors. Good. However, the same `notificationsModel` field is *also* mutated in-place by subsequent `loadNotifications` calls (line 224, `this.notificationsModel.loadData(...)`). On a transient failure during bootstrap and a subsequent successful poll, the model is updated correctly because it is the same reference — but the dual code paths (`getModel("notifications")` returning the field, vs. `setModel(notificationsModel)`) are harder to reason about than a single source of truth.
- **Suggested fix:** Always `setModel(this.notificationsModel, "notifications")` first (even before `loadNotifications` is awaited), then run `loadNotifications` independently. Removes the duplicated registration in the `catch` branch.

### 15. `BaseTableController.init` requires the table to be ready, but `onInit` of UI5 controllers runs before view rendering for derived classes — **Correctness** (low)

- **File:** `controller/BaseTable.controller.ts:36–50`, called from each derived controller’s `onInit`.
- **Problem:** `super.init(super.getView()?.byId("racesTable") as Table)` works in practice because `byId` resolves an aggregated control before the view is rendered, but only when the table has already been parsed. This relies on UI5’s declarative XML-view loading — fine, but the `as Table` cast hides a potential `undefined` if the id is misspelled (the next line `this.table.getBindingInfo(...)` would throw `Cannot read properties of undefined`). Several controllers use the same pattern.
- **Suggested fix:** Make `init` defensive:
  ```ts
  init(table: Table | undefined, channelId?: string): void {
      if (!table) {
          Log.error("BaseTableController.init: table is undefined");
          return;
      }
      this.table = table;
      ...
  }
  ```

### 16. `TimekeepingController._loadIcons` runs every controller construction and uses `sap.ui.require.toUrl` — **Minor / Correctness** (low)

- **File:** `controller/Timekeeping.controller.ts`, lines 45–55
- **Problem:** The TNT icon font is registered every time the controller is instantiated. `IconPool.registerFont` is idempotent, so this is benign, but the `sap.ui.require.toUrl(...)` call uses the global `sap.ui.require` — under TypeScript with `@openui5/types` this may trigger a typings issue (`Property 'require' does not exist on type ...`); confirm `tsconfig` does not silently widen.
- **Suggested fix:** Move icon registration to `Component.init` (executed once), and import `IconPool`/`sap/tnt/library` properly so `sap.ui.require.toUrl` is replaced by an `sap/ui/require` async import or a static import.

### 17. `Admin.controller.onPriorityChange` rebroadcasts whole notification — **Correctness** (low)

- **File:** `controller/Admin.controller.ts`, lines 93–101
- **Problem:** When the priority is changed from the inline `ComboBox`, the controller calls `updateNotification(notification.id, { ...notification, priority: newPriority })`. This sends a `PUT /api/notifications/{id}` containing *all* fields (title, text, priority, visible) — even if only priority changed. If a different administrator concurrently edits the title, this overwrite resurrects the stale title. Same issue in `onVisibilityChange`.
- **Suggested fix:** Either implement an HTTP `PATCH` semantic on the backend and send only the changed field, or read-modify-write with an `If-Match`/version field. As a minimum, pull the latest notification from the server immediately before sending the update.

### 18. `Admin.controller.dialogModel.priority` round-trips through `parseInt` — **Code Smell** (low)

- **File:** `controller/Admin.controller.ts`, lines 56–60, 96, 163, 189
- **Problem:** The priority is stored as a number in the JSON, displayed in a `Select` with string keys, and converted back via `Number.parseInt(... || "0", 10)`. Each call site repeats the same fallback (`"0"`). The `dialogModel.priority` is alternately a number (line 43) and a string (line 60 — `Number.parseInt(notification.priority || "0", 10)` *coerces* a number-typed value via `||`, which is fine for `0` but conflates "not set" and "value is zero" if the priority is ever explicitly `0`).
- **Suggested fix:** Type the dialog model with an interface, store priority as a `Priority` enum value (or strict `0|1|2|3`), and centralise the parse in a `Formatter.parsePriority` helper.

### 19. `Formatter` mixes German-only weekday/date strings with i18n keys — **Internationalisation** (medium)

- **File:** `webapp/model/Formatter.ts`, lines 559–577 (`weekdayLabel`), 537–558 (`dateLabel`), 55–85 (`dateTime`/`timestamp`)
- **Problem:** Many formatter outputs are hard-coded German (`"So"`, `"Mo"`, `"Di"` …) or German date order (`DD.MM.YYYY`). The i18n bundle exists for `de` and `en`, but Formatter outputs would not switch with locale. Similarly, `dateTime`/`timestamp` produce `DD.MM.YYYY HH:MM[:SS.sss]` regardless of locale; an English user would see the German date format.
- **Suggested fix:** Use UI5’s `sap.ui.core.format.DateFormat` (locale-aware) or `Intl.DateTimeFormat` keyed off `Core.getConfiguration().getLocale()`. For weekday labels, define i18n keys (`common.weekday.mon`, …) and look them up via `Formatter.bundle.getText`.

### 20. `Formatter.dayTimeIsoLabel` mixes UTC and local time — **Correctness Bug** (medium)

- **File:** `webapp/model/Formatter.ts`, lines 372–382
- **Problem:** This function uses `getUTCHours()` / `getUTCMinutes()` to format the time portion, while the **same** instance computes the weekday with `getDay()` (local). For a regatta near midnight in CET (UTC+1/+2), the weekday and the displayed time can refer to *different days*. Compare with `dateTime` (lines 55–68) which uses local for everything.
- **Suggested fix:** Pick one — almost certainly local time for the weekday label, since the regatta is held in a single time zone. Replace `getUTCHours/Minutes` with `getHours/Minutes`. If the backend really emits UTC and front-end formatting must remain UTC, then `getUTCDay()` should be used as well.

### 21. `Formatter` is a static-only class — **TS Idiom** (low)

- **File:** `webapp/model/Formatter.ts`
- **Problem:** The whole class consists of static methods, but every controller still imports it and assigns it to an instance property `readonly formatter: Formatter = Formatter;` so XML bindings can use `.formatter.method`. This is a known UI5/TS idiom, but the `: Formatter` annotation actually types the property as an *instance*, not a class — `Formatter` is the class object. Subtle, but `tsc --strict` allows it because the static side is structurally compatible. Consider documenting the convention in the file header.
- **Suggested fix:** Add a comment that explains the pattern, or convert `Formatter` to a plain `export const Formatter = { ... }` object. Alternatively, expose a singleton with `static readonly INSTANCE = new Formatter();` for clarity.

### 22. Race state enum and i18n labels are duplicated — **Maintainability** (low)

- **File:** `webapp/model/Formatter.ts`, lines 169–192 (`raceStateLabel`) vs. 415–425 (`heatStateLabel2`).
- **Problem:** `RaceState` and `HeatState` overlap in name and behaviour but have separate enums and separate label methods that map similar codes to similar i18n keys (`common.scheduled`, `heat.state.started`, …). `RaceState.Started`, `Unknown`, `Finished`, `PhotoFinish` all map to `"heat.state.started"` — a state-value to label collision that is non-obvious. Are race states really equivalent to heat states?
- **Suggested fix:** Audit the state semantics with the backend team. If race state is supposed to mirror heat state, drop one enum. Otherwise, distinguish the i18n keys (e.g., `race.state.unknown`).

### 23. `Formatter.crewLabel` uses `${crew.pos?.toString() || ""}` — **Minor** (low)

- **File:** `webapp/model/Formatter.ts`, line 306
- **Problem:** `pos` is typed `number | undefined`. `0?.toString()` returns `"0"` which is truthy, but if `pos` is `undefined`, the result is `""`. Reasonable. However, the construction `${position}: ${Formatter.athleteLabel(...)}` then yields `": Smith, ..."` when position is missing — a leading colon. The same `cox` flag could be used to produce a cleaner label.
- **Suggested fix:** When `position` is empty, omit the colon: `position ? \`${position}: ${name}\` : name`.

### 24. View XML files contain duplicated cell patterns — **Code Duplication** (low)

- **Files:** `view/RacesTable.view.xml` lines 73–105 (and similar in `HeatsTable.view.xml`).
- **Problem:** Each `ColumnListItem` has multiple `VBox` cells that show the same value with two `Text` controls — one with `class="cancelled"`, one without — toggled by `visible="{=${races>cancelled}}"`. This pattern repeats 5 times per row. The same outcome could be achieved with a single `Text` whose `class` is bound to the cancelled flag via a formatter.
- **Suggested fix:** Add a small formatter `Formatter.cancellableClass(item)` returning `"cancelled"` / `""` or `"boldCancelled"` / `"bold"`, and bind `class="{path:'races>',formatter:'.formatter.cancellableClass'}"` on a single `Text`. Halves the row markup and removes a class of "I forgot to update both" bugs.

### 25. `LoginPopover` `Fragment.load` uses `popover: any` — **Type Safety** (low)

- **File:** `controller/Launchpad.controller.ts`, lines 102–115
- **Problem:** The `Fragment.load` callback typed the popover as `any`. The wider `popoverPromise: Promise<ResponsivePopover>` is correct, but the inner reduction loses the type.
- **Suggested fix:** Type the inner `then` callback parameter explicitly as `ResponsivePopover` (or use `.then((popover: Awaited<typeof Fragment.load>) => ...)`).

### 26. `package.json` does not configure CI scripts (lint/typecheck) — **Tooling** (low)

- **File:** `static/package.json`
- **Problem:** The available scripts are `build`, `build:opt`, `ts-typecheck`, `lint`, `watch`. Good. But the project has no `test` script and no continuous-integration configuration committed (no `.github/workflows` referenced for the UI). `AGENTS.md` mentions `npm run ts-typecheck` for the UI, but there is no automated enforcement that this passes before merge.
- **Suggested fix:** Add a `test` script (initially a no-op or QUnit smoke test), wire `ts-typecheck` and `lint` into CI alongside the Rust workspace’s `cargo` checks.

### 27. No automated tests for UI logic — **Testing** (medium)

- **Files:** Entire UI module
- **Problem:** No QUnit, OPA5, Jest, or Vitest setup. The `Formatter` class — pure functions, easy to unit-test — has no tests. Routing patterns and dialog flows are unverified.
- **Suggested fix:** Add at minimum:
  - QUnit unit tests for `Formatter.*` (date/time formatting, label formatters with edge cases such as empty crews, undefined races),
  - OPA5 integration tests for the launchpad → races → details navigation flow,
  - a Vitest setup if you prefer a Node-based runner for the pure model layer.

### 28. `LoginPopover` keyboard handlers move focus instead of submitting — **UX / Minor** (low)

- **File:** `controller/Launchpad.controller.ts`, lines 52–60
- **Problem:** `onUserSubmit` focuses the password field (sensible) but `onPasswordSubmit` focuses the **login button** *and* invokes `performLogin()`. That is one operation too many: the focus shift to the button is invisible (the popover may already be closing), and `setFocus` on a button followed by an immediate `performLogin` triggers a redundant render.
- **Suggested fix:** Drop the `byId("login")?.focus()` call in `onPasswordSubmit`; just call `this.performLogin()`.

### 29. `Map` controller hard-codes regatta coordinates — **Maintainability** (low)

- **File:** `controller/Map.controller.ts`, lines 102–121
- **Problem:** The Sattelplatz, regatta office, finish, and start positions are hard-coded latitude/longitude pairs. If the regatta moves to a different venue, every position must be edited in code.
- **Suggested fix:** Move regatta-location data into the regatta JSON model from the backend (e.g. `regatta.locations: { office: [lat, lng], finish: [...], starts: [{distance: 1000, pos: [...]}, ...] }`) and consume it here. The popup labels can also become i18n keys then.

### 30. Notifications popup XHR success handler refreshes model unconditionally — **Minor** (low)

- **File:** `controller/Launchpad.controller.ts`, lines 67–80
- **Problem:** `onNotificationClose` POSTs `/api/notifications/{id}/read`. On success it calls `getComponentJSONModel("notifications")?.refresh()`. But the underlying *data* has already been updated optimistically by `removeItem`. If the POST fails (no `error` callback), the UI does not roll back: the item disappears locally yet the server still considers it unread. Next bootstrap cycle (60 s polling), the item will reappear, confusing the user.
- **Suggested fix:** Add an `error` handler that re-inserts the item or triggers a fresh fetch, and consider awaiting the POST before removing the item from the view.

### 31. Hard-coded German strings in views — **Internationalisation** (low)

- **Files:** `view/Map.view.xml` (likely), `controller/Map.controller.ts:86–88` (`"Regatta Orte"`, `"Vereine"`, `"Entfernung 250km"`, `"Sattelplatz"`, `"Regattabüro"`, `"Ziel"`, `"Start 1000m"`, `"Start 1500m"`)
- **Problem:** Although the i18n bundle exists, several user-visible strings are hard-coded German (overlay map labels, marker popups). The `i18n_en.properties` file therefore cannot fully translate the UI.
- **Suggested fix:** Move these to the i18n bundle (`map.layer.regatta`, `map.layer.clubs`, `map.layer.distance250`, `map.marker.office`, …).

### 32. `Component.getActiveRegatta` and `getFilters` race during bootstrap — **Correctness** (low)

- **File:** `Component.ts`, lines 121–146 (bootstrap), 199–219
- **Problem:** `bootstrap()` calls `Promise.all([this.getActiveRegatta(), this.getFilters()])`. `loadFilters()` itself calls `getActiveRegatta()` (line 212) before reading `this.regattaModel?.getData().id`. Because both top-level calls fire concurrently:
  - `getFilters` enters `loadFilters`, awaits its own `getActiveRegatta`,
  - the outer `getActiveRegatta` also runs, but they share the memoised promise — fine,
  - however `loadFilters` reads `this.regattaModel?.getData().id` immediately after the await — the field is set by `getActiveRegatta` resolution (line 46) which happens before this code, so it works, but only by accident of execution order.

  If a future refactor decouples `regattaModel` setting from `regattaModelPromise` resolution, the filters call will silently load with `regattaId = -1`.
- **Suggested fix:** Use the resolved `JSONModel` directly in `loadFilters`: `const regattaModel = await this.getActiveRegatta(); const regattaId = regattaModel.getData().id;` — without depending on `this.regattaModel` field.

### 33. `Component.exit` does not abort in-flight `loadData` — **Robustness** (low)

- **File:** `Component.ts`, lines 164–170
- **Problem:** `exit` clears the notifications interval, but in-flight `JSONModel.loadData` requests for regatta/filters/notifications continue. If the component is destroyed mid-bootstrap (rare in production, common in tests), the `then`/`catch` callbacks log into a destroyed component.
- **Suggested fix:** Track `AbortController`s for each load and abort them in `exit`. Alternatively, set a `destroyed` flag and short-circuit the `then` callbacks.

### 34. `BaseTable.controller.onFirstItemEvent` ignores its event-payload — **Minor** (info)

- **File:** `controller/BaseTable.controller.ts`, lines 70–102
- **Problem:** All four navigation handlers take `(channelId: string, eventId: string, parametersMap: any)` but use none of these parameters. The `parametersMap: any` is the typical UI5 event-bus signature. ESLint `no-unused-vars` would catch this.
- **Suggested fix:** Prefix unused parameters with `_` (`_channelId`, `_eventId`, `_parametersMap`) and update the eslint config so unused-vars allows leading-underscore names.

### 35. Persistent `LoginPopover` view-model leaks credentials across navigations — **Security** (low)

- **File:** `controller/Launchpad.controller.ts`, lines 22, 28
- **Problem:** `credentialsModel` is created once at controller construction and registered once on the view. The model lives as long as the view does. After successful login the password is reset (line 147), but if the user reopens the popover and types again then navigates away (without submitting), the password remains in the model. UI5 view caching means navigating back to the launchpad reveals it.
- **Suggested fix:** Reset `credentialsModel.setData({ username: "", password: "" })` whenever the popover is closed (event `afterClose`), regardless of how it was closed.

---

## Positive Observations

- **TypeScript with `strict: true`:** Strong baseline configuration that, once issues #3 / #25 are addressed, will catch most structural regressions at compile time. ✅
- **Declarative routing in `manifest.json`:** All routes / targets / patterns are in one place; level metadata is correctly used for transitions. ✅
- **Component lifecycle is well-thought-out:** Memoised promise pattern in `Component.ts` for regatta and filters models, including failure invalidation. The bootstrap separates static models, async models, and notification polling cleanly. ✅
- **Resource bundle injected into `Formatter`:** The previous deprecated synchronous `ResourceBundle.create({ async: false })` pattern has been replaced with explicit `Formatter.init(bundle)` + fallback to the i18n key — a clean fix to a known UI5 pitfall. ✅
- **Map popup builds a DOM tree, not an HTML string:** `Map.controller.buildClubPopup` constructs an `HTMLAnchorElement` with `createTextNode`, avoiding XSS through `innerHTML` — a previous review item visibly addressed. ✅
- **Event-bus subscription/unsubscription is symmetric:** `BaseTable.controller.onExit` unsubscribes everything `init` subscribed to, preventing memory leaks across view destroy/recreate cycles. ✅
- **Defensive `i18n` accessor in `Formatter`:** Returns the i18n key itself if the bundle has not been initialised, instead of throwing — matches UI5’s `{i18n>...}` binding fallback behaviour. ✅
- **`addEventDelegate(onBeforeShow/Hide)` used for keyboard shortcuts:** Correctly attaches/removes `keydown` listeners only while the detail view is visible, preventing global key capture from ghost views. ✅
- **i18n bundle is reasonably comprehensive:** Both `de` and `en` bundles cover the bulk of user-visible labels; `manifest.json` declares supported locales explicitly. ✅
- **WebSocket protocol auto-detection:** `Monitoring`/`Timekeeping` controllers correctly use `wss` over HTTPS and `ws` otherwise. ✅
- **Lazy fragment loading + caching:** `viewSettingsDialogs` map prevents re-loading the same fragment twice (modulo issue #9 about concurrent calls). ✅
- **Strict `_nav` invariants are tracked centrally:** `BaseTable.setCurrentItem` consistently sets `isFirst` / `isLast` so detail views can disable nav buttons accordingly (modulo issue #4 about where the data lives). ✅
- **Component-scoped JSONModels:** `race`, `heat`, `regatta`, `filters`, `device`, `identity`, `notifications` are all registered on the Component, making cross-view sharing explicit. ✅

---

## Suggested next steps (priority order)

1. **Authentication hardening (issue 2):** Make login flow `async/await`, add a global 401 handler, ensure passwords are wiped on close.
2. **Centralise REST access (issues 1, 5):** Create `ApiService` + `Endpoints`; migrate `Admin` and `Launchpad` off `$.ajax`.
3. **Replace toast-class hack (issue 6):** Switch to `MessageBox` / `MessageStrip` / `MessageToast.styleClass`.
4. **Type the JSON models (issues 3, 4):** Extend `model/types.ts` and remove `any` from controllers; move `_nav` to a dedicated nav model.
5. **Add Formatter unit tests (issue 27):** Quick win, surfaces the UTC/local bug (#20) and German hard-coding (#19).
6. **Decompose `BaseTable.controller` (issue 8):** Extract filter/sort/nav mixins; replace string-encoded filters with structured data.
7. **Reconnecting WebSocket + schema validation (issues 11, 12):** Especially important for timekeeping correctness.
8. **Clean up dead/duplicate files (issue 10):** Decide on `index_v2.html`; document or remove.
