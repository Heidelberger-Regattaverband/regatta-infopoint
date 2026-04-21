# Code Review: `infoportal` Crate

**Date:** 2026-04-20  
**Scope:** All source files in `infoportal/src/`

---

## Summary

The `infoportal` crate is a well-organized Actix-Web application serving as the HTTP frontend for the regatta system. It follows consistent patterns for REST endpoints, has good OpenAPI documentation via utoipa, and properly separates concerns between routing, authentication, and business logic. A few security and robustness issues were identified.

---

## Issues

### 1. `extract_credentials` is called twice in `authenticate`

- **File:** `infoportal/src/auth.rs`, lines 36–44
- **Problem:** `extract_credentials(req)` is called once to check for an existing pool (line 36), and if that fails, called again (line 43) to authenticate. The `Authorization` header is parsed and Base64-decoded twice for every new authentication.
- **Suggested fix:** Call `extract_credentials` once at the top and reuse the result:
  ```rust
  let (username, password) = extract_credentials(req)?;
  if let Some(pool) = pool_manager.get_pool(&username).await {
      return Some(pool);
  }
  pool_manager.create_pool(&username, &password).await.ok()
  ```

### 2. `get_timestamps` uses `.unwrap()` on pool connection

- **File:** `infoportal/src/http/rest_api/timekeeping.rs`, line 40
- **Problem:** `TiberiusPool::instance().get().await.unwrap()` will panic if the pool cannot provide a connection (e.g., pool exhausted, DB down). All other endpoints properly propagate errors via `match`.
- **Suggested fix:** Replace `.unwrap()` with proper error handling:
  ```rust
  let mut client = match TiberiusPool::instance().get().await {
      Ok(c) => c,
      Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
  };
  ```

### 3. SSL setup uses `.unwrap()` without context

- **File:** `infoportal/src/http/server.rs`, lines 23–25
- **Problem:** `SslAcceptor::mozilla_intermediate(...)`, `set_private_key_file(...)`, and `set_certificate_chain_file(...)` all use `.unwrap()`. If the SSL certificate files are missing or malformed, the error message will be unhelpful.
- **Suggested fix:** Use `.expect("descriptive message")` to provide context on failure, e.g., `.expect("Failed to load SSL private key from ssl/key.pem")`.

### 4. Error responses leak internal details

- **File:** Multiple REST API handlers (e.g., `misc.rs`, `race.rs`, `club.rs`, etc.)
- **Problem:** All error handlers return `HttpResponse::InternalServerError().body(e.to_string())`, which exposes raw database error messages (including SQL details, connection strings, etc.) to API consumers. This is a security concern.
- **Suggested fix:** Log the full error server-side and return a generic error message to the client:
  ```rust
  Err(e) => {
      tracing::error!("Query failed: {e}");
      HttpResponse::InternalServerError().json(serde_json::json!({"error": "Internal server error"}))
  }
  ```

### 5. `CacheQueryParams` is duplicated across multiple files

- **File:** `misc.rs`, `race.rs`, `club.rs`, `athlete.rs` — each defines its own `CacheQueryParams`
- **Problem:** The same struct with identical fields and derives is defined 4 times. This is unnecessary duplication.
- **Suggested fix:** Define `CacheQueryParams` once in `rest_api/mod.rs` (or a shared module) and import it in each handler file.

### 6. TLS encryption is disabled in `Config::db_config()`

- **File:** `infoportal/src/config.rs`, lines 55–56
- **Problem:** `EncryptionLevel::NotSupported` and `trust_cert()` are hardcoded, disabling TLS for database connections. This is a security concern for production deployments.
- **Suggested fix:** Make encryption configurable via CLI args / env vars, defaulting to encrypted connections. At minimum, add a `--db-no-tls` flag to explicitly opt out.

### 7. `get_timestamps` accesses `regatta.id` directly (pub field)

- **File:** `infoportal/src/http/rest_api/timekeeping.rs`, line 41
- **Problem:** The handler accesses `regatta.id` as a public field. While this works, it couples the handler to the internal structure of `Regatta`. Other endpoints use `regatta_id` from path parameters.
- **Suggested fix:** Minor concern. Consider whether the active regatta ID should be available via a method on `Aquarius` to avoid this coupling.

### 8. No authentication middleware — auth checks are manual in each handler

- **File:** `notification.rs`, `timekeeping.rs`, `authentication.rs`
- **Problem:** Each protected endpoint manually calls `auth::authenticate(&req).await` and returns 401. This is repetitive and error-prone — it's easy to forget the check when adding new endpoints.
- **Suggested fix:** Consider implementing an Actix-Web middleware or extractor for authentication that can be applied to a scope, e.g., wrap the protected routes in a scope with an auth middleware.

### 9. `FORCE_CACHE` constant is defined but unused

- **File:** `infoportal/src/http/rest_api.rs`, line 12
- **Problem:** `const FORCE_CACHE: &str = "force"` is defined but never referenced. The `force` parameter is handled via `CacheQueryParams` deserialization instead.
- **Suggested fix:** Remove the unused constant.

### 10. No unit or integration tests

- **File:** Entire crate
- **Problem:** There are no `#[cfg(test)]` modules or test files. The HTTP handlers, authentication logic, and configuration parsing could all benefit from tests.
- **Suggested fix:** Add tests for `extract_credentials` (pure function, easy to test), `Config` parsing, and integration tests for the API endpoints using Actix-Web's test utilities.

---

## Positive Observations

- **Clean REST API design:** Consistent URL patterns following RESTful conventions (`/regattas/{id}/clubs/{id}/entries`).
- **OpenAPI documentation:** All endpoints have `#[utoipa::path]` annotations with proper tags, response types, and parameter descriptions.
- **Proper use of Actix-Web extractors:** `Data`, `Path`, `Query`, and `Json` extractors are used idiomatically throughout.
- **Health endpoint:** Comprehensive monitoring endpoint exposing version, pool state, memory usage, and cache statistics.
- **Configuration via clap:** All settings are configurable via CLI args or environment variables with sensible defaults.
- **Build info:** `built` crate provides compile-time metadata (version, git hash, build time) exposed in the health endpoint.