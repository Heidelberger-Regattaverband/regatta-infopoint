# Code Review: `infoportal` Crate

**Updated:** 2026-08-24 (original: 2026-04-20)
**Scope:** All source files in `infoportal/src/`

---

## Open Issues

### HIGH-4 — DB TLS enabled but certificate validation bypassed ✅ NOT AN ISSUE

**File:** `infoportal/src/config.rs`, lines 106–109

```rust
if self.db_encryption {
    config.encryption(EncryptionLevel::Required);
    config.trust_cert();  // ← disables cert validation
```

`trust_cert()` accepts any certificate, including adversarially crafted ones. Enabling TLS while calling `trust_cert()` provides no MITM protection.

**Suggested fix:** Remove the unconditional `trust_cert()` call. Introduce a separate `DB_TRUST_CERT=false` env var for development self-signed certs, documented as unsafe for production.

---

### MEDIUM-1 — WebSocket timekeeping errors expose internal details ✅ FIXED

**File:** `infoportal/src/http/rest_api/timekeeping.rs`, lines 245, 252, 281, 309, 313, 363

Raw tiberius/SQL errors are formatted into `ServerEvent::Error` and sent to the WebSocket client, potentially leaking table names, column names, or connection details.

**Suggested fix:**
```rust
.map_err(|err| {
    error!(?err, "Failed to add start timestamp");
    "Failed to add start timestamp".to_string()
})?
```

---

### MEDIUM-2 — `thread::sleep` inside a global `Mutex` lock in async context

**File:** `infoportal/src/http/monitoring.rs`, lines 78–89

On the first call to `get_cpu_and_memory()`, `thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL)` (200 ms) is called while holding the global `CACHED` mutex. All concurrent callers block for 200 ms.

**Suggested fix:** Release the lock before sleeping, then re-acquire to store the result. Or pre-initialize the cache on server startup.

---

### MEDIUM-3 — Session secret key is ephemeral — all sessions invalidated on restart

**File:** `infoportal/src/http/server.rs`, line 65

```rust
let secret_key = Key::generate();
```

Every restart invalidates all active sessions. For rolling restarts or auto-scaling this silently logs out all users.

**Suggested fix:** Load the key from a stable env var `SESSION_SECRET`, falling back to `Key::generate()` with a warning in development only.

---

### MEDIUM-4 — Admin scope determined by hardcoded username list

**File:** `infoportal/src/auth.rs`, lines 38–43

```rust
"sa" | "admin" => Scope::Admin,
_ => Scope::User,
```

Any SQL Server account named `sa` or `admin` gets web-app admin access, regardless of actual DB permissions. Any other name gets `Scope::User` regardless of intent.

**Suggested fix:** Use a DB role or configurable env var `ADMIN_USERNAMES` rather than matching on account names.

---

### MEDIUM-5 — `create_app_data().await.unwrap()` panics on startup DB failure

**File:** `infoportal/src/http/server.rs`, line 63

The `start()` function returns `IoResult<()>`, so propagating with `?` is idiomatic and correct. Also `TimeStrip::load(...).await.unwrap()` on line 166 of `timekeeping.rs` panics when the DB is unavailable at WebSocket connection time.

**Suggested fix:**
```rust
let aquarius = create_app_data().await
    .map_err(|e| std::io::Error::other(e.to_string()))?;
```

---

### MEDIUM-6 — `TtlExtensionPolicy::OnEveryRequest` effectively disables session expiry for active users

**File:** `infoportal/src/http/server.rs`, lines 178–182

Every HTTP request (including static asset requests) refreshes the session TTL, making the 2-day TTL meaningless for any browser tab left open.

**Suggested fix:** Use `TtlExtensionPolicy::OnStateChanges` or apply the session middleware only to API routes.

---

### MEDIUM-7 — `notification_read` can bloat session cookie without bound

**File:** `infoportal/src/http/rest_api/notification.rs`, lines 198–202

`POST /api/notifications/{id}/read` requires no authentication and inserts a new session entry per notification ID. A client can call this with thousands of IDs, exceeding the ~4 KB cookie limit and breaking the session.

**Suggested fix:** Cap the number of read markers per session, or periodically prune markers for no-longer-visible notifications.

---

### LOW-1 — `_identity: Identity` is a fragile implicit auth pattern (still open from #8)

**Files:** `infoportal/src/http/rest_api/misc.rs`, lines 29, 51; `infoportal/src/http/rest_api/monitoring.rs`, line 116

The underscore prefix signals "unused" to readers and linters, obscuring the security contract. A future developer might change it to `Option<Identity>` and inadvertently remove the auth guard.

**Suggested fix:** Introduce a typed `AuthenticatedUser` extractor that returns 401 when unauthenticated, or apply auth at the scope level. At minimum add a comment: `// Auth guard: returns 401 if no active session`.

---

### LOW-2 — `serde_json::to_string(...).unwrap_or_default()` silently swallows serialization errors

**Files:** `infoportal/src/http/rest_api/timekeeping.rs`, line 226; `infoportal/src/http/rest_api/monitoring.rs`, line 106

Serialization failures send an empty string to the WebSocket client with no log entry.

**Suggested fix:**
```rust
match serde_json::to_string(&event) {
    Ok(json) => ctx.text(json),
    Err(err) => error!(?err, "Failed to serialize WebSocket event"),
}
```

---

### LOW-3 — `worker_count.lock().unwrap()` panics on mutex poisoning

**File:** `infoportal/src/http/server.rs`, line 74

**Suggested fix:** Use `.unwrap_or_else(|e| e.into_inner())` for poison recovery.

---

### LOW-4 — TLS falls back to HTTP-only silently

**File:** `infoportal/src/http/server.rs`, lines 109–113

Missing or unreadable cert/key files cause the server to start HTTP-only with only a `warn!` log.

**Suggested fix:** Consider making TLS mandatory by default and requiring an explicit `HTTP_ONLY=true` env var to allow plain HTTP.

---

### LOW-5 — Swagger UI publicly accessible

**File:** `infoportal/src/http/api_doc.rs`, lines 42–46

`/swagger-ui/` and `/api-docs/openapi.json` expose the full API schema to unauthenticated clients. Low risk for an internal system but aids reconnaissance if internet-facing.

**Suggested fix:** Remove from production builds (`#[cfg(debug_assertions)]`) or restrict to authenticated scope.

---

### LOW-6 — Notification content has no length validation

**File:** `infoportal/src/http/rest_api/notification.rs`, lines 99–111

`title` is validated for non-empty but has no max-length constraint. An authenticated user could store arbitrarily large strings.

**Suggested fix:** Add explicit length limits matching the DB column constraints.
