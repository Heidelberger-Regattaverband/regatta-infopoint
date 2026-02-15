# Custom Database Connections for Authenticated Users

This document describes how to use per-user database connections in the Regatta Infopoint application.

## Overview

The application now supports maintaining separate database connection pools for each authenticated user. This allows users to connect to the database with their own credentials, enabling fine-grained access control and auditing.

## Architecture

### Components

1. **UserDbCredentials**: Struct representing user-specific database credentials
2. **UserPoolManager**: Manager that maintains a cache of connection pools per user
3. **User Authentication**: Updated to accept optional database credentials during login
4. **Helper Functions**: Utilities to get user-specific database pools and cleanup on logout

### How It Works

1. When a user logs in, they can optionally provide database credentials (`db_username` and `db_password`)
2. These credentials are stored in the user's session (`UserInfo`)
3. When the user makes API requests, the application can retrieve their dedicated connection pool
4. Connection pools are cached for performance and reused across requests
5. When the user logs out, their connection pool is cleaned up

## Usage

### Login with Custom Database Credentials

```bash
# Login with custom DB credentials
curl -X POST http://localhost:8080/api/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "user",
    "password": "user",
    "db_username": "db_user_readonly",
    "db_password": "readonly_pass"
  }'
```

### Using User-Specific Pools in API Handlers

In your API handlers, you can get the user-specific database pool:

```rust
use crate::http::{get_user_pool, server::AppState};
use actix_web::{get, web, HttpResponse, Responder};

#[get("/api/my-data")]
pub async fn get_my_data(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    // Get authenticated user
    let user_info = state.auth.get_user_from_request(&req).await
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;
    
    // Get user's database pool (or default application pool)
    let pool = get_user_pool(
        &user_info,
        &state.user_pool_manager,
        state.aquarius.pool.clone()
    ).await?;
    
    // Use the pool to execute queries
    let mut conn = pool.get().await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    
    // Execute your queries with the user's credentials...
    
    Ok(HttpResponse::Ok().json("Data"))
}
```

## Benefits

1. **Security**: Each user connects with their own database credentials
2. **Auditing**: Database can track which user performed which operations
3. **Access Control**: Database-level permissions can be enforced per user
4. **Performance**: Connection pools are reused across requests
5. **Flexibility**: Falls back to application credentials if user doesn't provide custom ones

## Configuration

The base database configuration is still loaded from the application config file. User-specific credentials override only the username and password, while keeping the same host, port, and database name.

## Cleanup

When a user logs out, their connection pool is automatically cleaned up to free resources. The cleanup happens in the `/api/logout` endpoint.

## Security Considerations

1. **Credential Storage**: User database credentials are stored in memory only and are never persisted
2. **Transport Security**: Always use HTTPS to transmit database credentials during login
3. **Validation**: Ensure user-provided credentials are validated against your authentication system
4. **Database Permissions**: Configure database user permissions appropriately to limit access

## Example Scenarios

### Scenario 1: Read-Only User
```json
{
  "username": "viewer",
  "password": "viewer123",
  "db_username": "readonly_user",
  "db_password": "readonly_pass"
}
```

### Scenario 2: Admin User with Full Access
```json
{
  "username": "admin",
  "password": "admin123",
  "db_username": "db_admin",
  "db_password": "admin_pass"
}
```

### Scenario 3: Using Application Default
```json
{
  "username": "user",
  "password": "user123"
  // No db_username/db_password - uses application's default credentials
}
```

## Monitoring

You can monitor the number of active user connection pools:

```rust
let pool_count = state.user_pool_manager.pool_count().await;
println!("Active user pools: {}", pool_count);
```

## Implementation Notes

- Connection pools are created lazily on first access
- Pools are cached and reused for the same credentials
- Thread-safe implementation using `Arc<RwLock<HashMap>>`
- Automatic cleanup on logout prevents resource leaks