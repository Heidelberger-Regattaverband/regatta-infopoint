# Configuration Code Analysis and Improvements

## Current Issues Identified

### 1. **Inconsistent Constant Usage**
- `DB_POOL_MAX_SIZE` constant is defined but not used consistently throughout the code
- Magic strings are used instead of constants for environment variable names

### 2. **Performance Issues**
- Unnecessary string cloning in getter methods (`get_http_bind`, `get_https_bind`)
- Using `&String` instead of `&str` in `get_db_config_for_user` method
- Multiple `to_string()` calls for logging that could be optimized

### 3. **Design Issues**
- Logging mixed with getter logic - getters should be pure
- Validation logic could be better organized
- Some methods are doing too much (e.g., `init` method is very long)

### 4. **Documentation Issues**
- Comment mentions `HTTPS_KEY_Path` but code uses `HTTPS_KEY_PATH`
- Some defaults could be better documented

## Suggested Improvements

### 1. **Constants Module**
Extract all environment variable names and defaults into constants:

```rust
mod constants {
    // HTTP Configuration
    pub const HTTP_BIND: &str = "HTTP_BIND";
    pub const HTTP_PORT: &str = "HTTP_PORT";
    pub const HTTP_APP_CONTENT_PATH: &str = "HTTP_APP_CONTENT_PATH";
    pub const HTTP_WORKERS: &str = "HTTP_WORKERS";
    
    // HTTPS Configuration
    pub const HTTPS_BIND: &str = "HTTPS_BIND";
    pub const HTTPS_PORT: &str = "HTTPS_PORT";
    pub const HTTPS_CERT_PATH: &str = "HTTPS_CERT_PATH";
    pub const HTTPS_KEY_PATH: &str = "HTTPS_KEY_PATH";
    
    // Rate Limiting
    pub const HTTP_RL_MAX_REQUESTS: &str = "HTTP_RL_MAX_REQUESTS";
    pub const HTTP_RL_INTERVAL: &str = "HTTP_RL_INTERVAL";
    
    // Database Configuration
    pub const DB_HOST: &str = "DB_HOST";
    pub const DB_PORT: &str = "DB_PORT";
    pub const DB_NAME: &str = "DB_NAME";
    pub const DB_USER: &str = "DB_USER";
    pub const DB_PASSWORD: &str = "DB_PASSWORD";
    pub const DB_ENCRYPTION: &str = "DB_ENCRYPTION";
    pub const DB_POOL_MAX_SIZE: &str = "DB_POOL_MAX_SIZE";
    pub const DB_POOL_MIN_IDLE: &str = "DB_POOL_MIN_IDLE";
    
    // Application Configuration
    pub const ACTIVE_REGATTA_ID: &str = "ACTIVE_REGATTA_ID";
    pub const CACHE_TTL: &str = "CACHE_TTL";
    
    // Default Values
    pub const DEFAULT_HTTP_BIND: &str = "0.0.0.0";
    pub const DEFAULT_HTTP_PORT: &str = "8080";
    pub const DEFAULT_HTTPS_BIND: &str = "0.0.0.0";
    pub const DEFAULT_HTTPS_PORT: &str = "8443";
    pub const DEFAULT_HTTPS_CERT_PATH: &str = "./ssl/cert.pem";
    pub const DEFAULT_HTTPS_KEY_PATH: &str = "./ssl/key.pem";
    pub const DEFAULT_HTTP_APP_CONTENT_PATH: &str = "./static/dist";
    pub const DEFAULT_HTTP_RL_MAX_REQUESTS: &str = "500";
    pub const DEFAULT_HTTP_RL_INTERVAL: &str = "600";
    pub const DEFAULT_DB_PORT: &str = "1433";
    pub const DEFAULT_DB_ENCRYPTION: &str = "false";
    pub const DEFAULT_DB_POOL_MAX_SIZE: &str = "100";
    pub const DEFAULT_DB_POOL_MIN_IDLE: &str = "30";
    pub const DEFAULT_CACHE_TTL: &str = "30";
    
    // Validation Limits
    pub const MAX_CACHE_TTL: u64 = 3600; // 1 hour
}
```

### 2. **Improved Getter Methods**
Remove logging from getters and return references instead of cloning:

```rust
impl Config {
    /// Returns the HTTP binding configuration (host, port)
    pub fn http_bind(&self) -> (&str, u16) {
        (&self.http_bind, self.http_port)
    }

    /// Returns the HTTPS binding configuration (host, port)
    pub fn https_bind(&self) -> (&str, u16) {
        (&self.https_bind, self.https_port)
    }

    /// Returns the rate limiter configuration (max_requests, interval_seconds)
    pub fn rate_limiter_config(&self) -> (u64, u64) {
        (self.http_rl_max_requests, self.http_rl_interval)
    }
    
    /// Log server binding information (separate from getters)
    pub fn log_server_config(&self) {
        info!(
            "HTTP server listening on: {}:{}",
            self.http_bind.bold(),
            self.http_port.to_string().bold()
        );
        info!(
            "HTTPS server listening on: {}:{}",
            self.https_bind.bold(),
            self.https_port.to_string().bold()
        );
        info!(
            "Rate limiter: {} requests per {} seconds",
            self.http_rl_max_requests.to_string().bold(),
            self.http_rl_interval.to_string().bold()
        );
    }
}
```

### 3. **Better Parameter Types**
Use `&str` instead of `&String` for better performance:

```rust
/// Returns the database configuration for a specific user
pub fn get_db_config_for_user(&self, user: &str, password: &str) -> TiberiusConfig {
    let mut config = TiberiusConfig::new();
    config.host(&self.db_host);
    config.port(self.db_port);
    config.database(&self.db_name);
    config.authentication(AuthMethod::sql_server(user, password));
    
    if self.db_encryption {
        config.encryption(EncryptionLevel::Required);
        config.trust_cert();
    } else {
        config.encryption(EncryptionLevel::NotSupported);
    }
    
    config
}
```

### 4. **Modular Initialization**
Break down the large `init` method into smaller, focused methods:

```rust
impl Config {
    fn init() -> Result<Self, ConfigError> {
        dotenv().ok();
        env_logger::init();
        
        Self::log_build_info();
        
        let http_config = Self::load_http_config()?;
        let https_config = Self::load_https_config()?;
        let db_config = Self::load_db_config()?;
        let app_config = Self::load_app_config()?;
        
        Ok(Config {
            // HTTP
            http_bind: http_config.bind,
            http_port: http_config.port,
            http_rl_max_requests: http_config.rl_max_requests,
            http_rl_interval: http_config.rl_interval,
            http_workers: http_config.workers,
            http_app_content_path: http_config.app_content_path,
            
            // HTTPS
            https_bind: https_config.bind,
            https_port: https_config.port,
            https_cert_path: https_config.cert_path,
            https_key_path: https_config.key_path,
            
            // Database
            db_host: db_config.host,
            db_port: db_config.port,
            db_name: db_config.name,
            db_user: db_config.user,
            db_password: db_config.password,
            db_encryption: db_config.encryption,
            db_pool_max_size: db_config.pool_max_size,
            db_pool_min_idle: db_config.pool_min_idle,
            
            // Application
            active_regatta_id: app_config.active_regatta_id,
            cache_ttl: app_config.cache_ttl,
        })
    }
    
    fn log_build_info() {
        info!(
            "Build: time '{}', commit '{}', head_ref '{}'",
            built_info::BUILT_TIME_UTC.bold(),
            built_info::GIT_COMMIT_HASH.unwrap_or_default().bold(),
            built_info::GIT_HEAD_REF.unwrap_or_default().bold()
        );
    }
    
    fn load_http_config() -> Result<HttpConfig, ConfigError> {
        let bind = env::var(constants::HTTP_BIND)
            .unwrap_or_else(|_| constants::DEFAULT_HTTP_BIND.to_string());
        let port = Self::parse_env_var(constants::HTTP_PORT, constants::DEFAULT_HTTP_PORT)?;
        let app_content_path = env::var(constants::HTTP_APP_CONTENT_PATH)
            .unwrap_or_else(|_| constants::DEFAULT_HTTP_APP_CONTENT_PATH.to_string());
        let rl_max_requests = Self::parse_env_var(constants::HTTP_RL_MAX_REQUESTS, constants::DEFAULT_HTTP_RL_MAX_REQUESTS)?;
        let rl_interval = Self::parse_env_var(constants::HTTP_RL_INTERVAL, constants::DEFAULT_HTTP_RL_INTERVAL)?;
        let workers = Self::parse_optional_env_var(constants::HTTP_WORKERS)?;
        
        info!("Serving static content from: {}", app_content_path.bold());
        
        Ok(HttpConfig {
            bind,
            port,
            rl_max_requests,
            rl_interval,
            workers,
            app_content_path,
        })
    }
}

// Helper structs for modular configuration loading
struct HttpConfig {
    bind: String,
    port: u16,
    rl_max_requests: u64,
    rl_interval: u64,
    workers: Option<usize>,
    app_content_path: String,
}

struct HttpsConfig {
    bind: String,
    port: u16,
    cert_path: String,
    key_path: String,
}

struct DbConfig {
    host: String,
    port: u16,
    name: String,
    user: String,
    password: String,
    encryption: bool,
    pool_max_size: u32,
    pool_min_idle: u32,
}

struct AppConfig {
    active_regatta_id: Option<i32>,
    cache_ttl: u64,
}
```

### 5. **Enhanced Validation**
Create a dedicated validation module:

```rust
mod validation {
    use super::{ConfigError, constants};
    
    pub fn validate_db_config(
        host: &str,
        port: u16,
        pool_max_size: u32,
        pool_min_idle: u32,
    ) -> Result<(), ConfigError> {
        validate_db_host(host)?;
        validate_db_port(port)?;
        validate_db_pool_config(pool_max_size, pool_min_idle)?;
        Ok(())
    }
    
    fn validate_db_host(host: &str) -> Result<(), ConfigError> {
        if host.trim().is_empty() {
            return Err(ConfigError::InvalidValue {
                var_name: constants::DB_HOST.to_string(),
                reason: "Database host cannot be empty".to_string(),
            });
        }
        Ok(())
    }
    
    fn validate_db_port(port: u16) -> Result<(), ConfigError> {
        if port == 0 {
            return Err(ConfigError::InvalidValue {
                var_name: constants::DB_PORT.to_string(),
                reason: "Database port cannot be 0".to_string(),
            });
        }
        Ok(())
    }
    
    fn validate_db_pool_config(max_size: u32, min_idle: u32) -> Result<(), ConfigError> {
        if max_size == 0 {
            return Err(ConfigError::InvalidValue {
                var_name: constants::DB_POOL_MAX_SIZE.to_string(),
                reason: "Database pool max size must be greater than 0".to_string(),
            });
        }
        
        if min_idle > max_size {
            return Err(ConfigError::InvalidValue {
                var_name: constants::DB_POOL_MIN_IDLE.to_string(),
                reason: format!(
                    "Database pool min idle ({}) cannot exceed max size ({})",
                    min_idle, max_size
                ),
            });
        }
        
        Ok(())
    }
    
    pub fn validate_cache_ttl(ttl: u64) -> Result<(), ConfigError> {
        if ttl == 0 {
            return Err(ConfigError::InvalidValue {
                var_name: constants::CACHE_TTL.to_string(),
                reason: "Cache TTL must be greater than 0 seconds".to_string(),
            });
        }
        
        if ttl > constants::MAX_CACHE_TTL {
            return Err(ConfigError::InvalidValue {
                var_name: constants::CACHE_TTL.to_string(),
                reason: format!(
                    "Cache TTL ({} seconds) exceeds maximum recommended value of {} seconds",
                    ttl, constants::MAX_CACHE_TTL
                ),
            });
        }
        
        Ok(())
    }
}
```

## Key Benefits of These Improvements

1. **Better Performance**: 
   - Eliminates unnecessary string cloning
   - Uses string slices where appropriate
   - Reduces memory allocations

2. **Improved Maintainability**:
   - Constants reduce magic strings
   - Modular initialization is easier to understand and modify
   - Separation of concerns (logging vs getters)

3. **Enhanced Error Handling**:
   - More specific validation functions
   - Better error messages with context

4. **Better Testing**:
   - Smaller, focused functions are easier to unit test
   - Validation logic can be tested independently

5. **Cleaner Code**:
   - Follows single responsibility principle
   - Consistent naming and organization
   - Better documentation alignment with code

## Implementation Priority

1. **High Priority**: Fix parameter types (`&String` -> `&str`)
2. **Medium Priority**: Extract constants and modularize initialization
3. **Low Priority**: Separate logging from getters (breaking change)

These improvements maintain backward compatibility while significantly improving code quality, performance, and maintainability.
