# Implementation Guide: Custom Database Connections for Authenticated Users

This guide shows you how to integrate the custom database connection feature that has been implemented.

## What Was Implemented

The following components have been added to support per-user database connections:

1. **`infoportal/src/db/user_pool.rs`** - Core implementation
   - `UserDbCredentials`: Struct for user database credentials
   - `DbConfig`: Database configuration struct
   - `UserPoolManager`: Manages per-user connection pools

2. **`infoportal/src/http/user_db_middleware.rs`** - Helper functions
   - `get_user_pool()`: Get a user's database pool
   - `cleanup_user_pool()`: Clean up on logout

3. **`db/src/tiberius/pool.rs`** - Enhanced with `from_pool()` method
4. **`db/src/lib.rs`** - Exports `bb8` for pool creation

## Integration Steps

### Step 1: Update Authentication to Store DB Credentials

Modify `infoportal/src/http/auth.rs` to store database credentials:

```rust
use crate::db::UserDbCredentials;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct User {
    pub(crate) username: String,
    scope: Scope,
    #[serde(skip)]
    pub(crate) db_credentials: Option<UserDbCredentials>,
}
```

### Step 2: Update Login to Accept DB Credentials

Modify `infoportal/src/http/rest_api/authentication.rs`:

```rust
#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
    #[serde(default)]
    db_username: Option<String>,
    #[serde(default)]
    db_password: Option<String>,
}

#[post("/login")]
async fn login(credentials: Json<LoginRequest>, request: HttpRequest) -> Result<impl Responder, Error> {
    // Create DB credentials if provided
    let db_credentials = if let (Some(db_user), Some(db_pass)) = 
        (credentials.db_username.clone(), credentials.db_password.clone()) {
        Some(UserDbCredentials {
            username: db_user,
            password: db_pass,
        })
    } else {
        None
    };
    
    // Store in session for later use
    // ... rest of authentication logic
}
```

### Step 3: Initialize UserPoolManager in Server

Modify `infoportal/src/http/server.rs` or wherever you initialize your app state:

```rust
use crate::db::{user_pool::{DbConfig, UserPoolManager}, UserDbCredentials};

// Create the configuration
let db_config = DbConfig {
    host: CONFIG.db_host.clone(),
    port: CONFIG.db_port,
    database: CONFIG.db_name.clone(),
    encryption: CONFIG.db_encryption,
    pool_max_size: CONFIG.db_pool_max_size,
    pool_min_idle: CONFIG.db_pool_min_idle,
};

// Create the user pool manager
let user_pool_manager = UserPoolManager::new(db_config);

// Add to your app state
let app_data = web::Data::new(AppState {
    aquarius,
    user_pool_manager,
    // ... other fields
});
```

### Step 4: Use Per-User Pools in API Handlers

Example of using user-specific database pools:

```rust
use crate::http::user_db_middleware::get_user_pool;

#[get("/api/my-data")]
pub async fn get_my_data(
    identity: Option<Identity>,
    state: web::Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    // Get user info from session
    let user_info = identity
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;
    
    // Get user's pool or default application pool
    let pool = get_user_pool(
        &user_info,
        &state.user_pool_manager,
        Arc::new(TiberiusPool::instance().clone()) // or state.aquarius.pool
    ).await?;
    
    // Use the pool
    let mut conn = pool.get().await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    
    // Execute queries...
    
    Ok(HttpResponse::Ok().json("data"))
}
```

### Step 5: Clean Up on Logout

Update logout handler:

```rust
use crate::http::user_db_middleware::cleanup_user_pool;

#[post("/logout")]
async fn logout(
    user: Identity,
    state: web::Data<AppState>,
) -> impl Responder {
    // Get user info before logout
    if let Ok(username) = user.id() {
        // If user has custom credentials, clean up their pool
        // You'll need to store the full UserInfo in session to get credentials
        cleanup_user_pool(&user_info, &state.user_pool_manager).await;
    }
    
    user.logout();
    HttpResponse::NoContent()
}
```

## Usage Example

### Login with Custom Database Credentials

```bash
curl -X POST http://localhost:8080/api/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "app_user",
    "password": "app_password",
    "db_username": "readonly_db_user",
    "db_password": "readonly_pass"
  }'
```

### Login Without Custom Credentials (Uses Application Default)

```bash
curl -X POST http://localhost:8080/api/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "app_user",
    "password": "app_password"
  }'
```

## Benefits

1. **Security**: Users connect with their own database credentials
2. **Auditing**: Database tracks which user performed operations
3. **Access Control**: Database-level permissions enforced per user
4. **Performance**: Connection pools cached and reused
5. **Flexibility**: Falls back to application credentials if not provided

## Notes

- The implementation is currently not wired up to avoid breaking existing code
- You need to modify your authentication flow to store and retrieve user database credentials
- Consider using actix-session to store UserDbCredentials in the session
- The `CUSTOM_DB_CONNECTIONS.md` file contains detailed usage documentation

## Security Considerations

1. Database credentials are stored in memory only (never persisted)
2. Always use HTTPS to transmit credentials
3. Validate credentials before creating pools
4. Set appropriate database permissions for each user
5. Consider implementing credential rotation