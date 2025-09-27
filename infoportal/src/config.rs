use crate::built_info;
use colored::Colorize;
use dotenv::dotenv;
use log::{error, info};
use std::{
    env,
    error::Error,
    fmt::{self, Display},
    str::FromStr,
    sync::OnceLock,
};
use tiberius::{AuthMethod, Config as TiberiusConfig, EncryptionLevel};

static CONFIG: OnceLock<Config> = OnceLock::new();

/// Configuration error type for better error handling
#[derive(Debug)]
pub enum ConfigError {
    /// Environment variable parsing error
    ParseError {
        var_name: String,
        value: String,
        error: String,
    },
    /// Missing required environment variable
    MissingRequired(String),
    /// Invalid configuration value
    InvalidValue { var_name: String, reason: String },
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::ParseError { var_name, value, error } => {
                write!(
                    f,
                    "Failed to parse environment variable '{}' with value '{}': {}",
                    var_name, value, error
                )
            }
            ConfigError::MissingRequired(var_name) => {
                write!(f, "Required environment variable '{}' is not set", var_name)
            }
            ConfigError::InvalidValue { var_name, reason } => {
                write!(f, "Invalid value for environment variable '{}': {}", var_name, reason)
            }
        }
    }
}

impl Error for ConfigError {}

/// The configuration of the server. The configuration is read from the environment.
/// The configuration is a singleton and initialized once. The configuration can be accessed by calling `Config::get()`.
pub struct Config {
    /// The IP address the HTTP server is listening on. Defaults to `0.0.0.0`.
    /// The IP address can be set by setting the environment variable `HTTP_BIND`.
    pub http_bind: String,
    /// The port the HTTP server is listening on. Defaults to `8080`.
    /// The port can be set by setting the environment variable `HTTP_PORT`.
    pub http_port: u16,
    /// The IP address the HTTPS server is listening on. Defaults to `0.0.0.0`
    /// The IP address can be set by setting the environment variable `HTTPS_BIND`.
    pub https_bind: String,
    /// The port the HTTPS server is listening on. Defaults to `8443`.
    /// The port can be set by setting the environment variable `HTTPS_PORT`.
    pub https_port: u16,
    /// The path to the HTTPS certificate. Defaults to `./ssl/cert.pem`.
    /// The path can be set by setting the environment variable `HTTPS_CERT_PATH`.
    pub https_cert_path: String,
    /// The path to the HTTPS key. Defaults to `./ssl/key.pem`.
    /// The path can be set by setting the environment variable `HTTPS_KEY_PATH`.
    pub https_key_path: String,
    /// The maximum number of requests per interval.
    /// The maximum number of requests can be set by setting the environment variable `HTTP_RL_MAX_REQUESTS`.
    pub http_rl_max_requests: u64,
    /// The rate interval in seconds. The rate interval can be set by setting the environment variable `HTTP_RL_INTERVAL`.
    pub http_rl_interval: u64,
    /// The number of HTTP workers. The number of HTTP workers can be set by setting the environment variable `HTTP_WORKERS`.
    pub http_workers: Option<usize>,
    /// The path to the static application content that is delivered to the browser. Defaults to `./static/dist`.
    /// The path can be set by setting the environment variable `HTTP_APP_CONTENT_PATH`.
    pub http_app_content_path: String,
    /// The database host. The database host can be set by setting the environment variable `DB_HOST`.
    pub db_host: String,
    /// The database port. The database port can be set by setting the environment variable `DB_PORT`.
    pub db_port: u16,
    /// The database name. The database name can be set by setting the environment variable `DB_NAME`.
    pub db_name: String,
    /// The database user. The database user can be set by setting the environment variable `DB_USER`.
    pub db_user: String,
    /// The database password. The database password can be set by setting the environment variable `DB_PASSWORD`.
    pub db_password: String,
    /// Whether the database connection should be encrypted. Defaults to `false`.
    /// The database encryption can be set by setting the environment variable `DB_ENCRYPTION`.
    pub db_encryption: bool,
    /// The maximum number of connections in the database pool.
    /// The maximum number of connections can be set by setting the environment variable `DB_POOL_MAX_SIZE`.
    pub db_pool_max_size: u32,
    /// The minimum number of idle connections in the database pool.
    /// The minimum number of idle connections can be set by setting the environment variable `DB_POOL_MIN_IDLE`.
    pub db_pool_min_idle: u32,
    /// The ID of the active regatta. The ID of the active regatta can be set by setting the environment variable `ACTIVE_REGATTA_ID`.
    pub active_regatta_id: Option<i32>,
    /// The cache TTL in seconds. The cache TTL can be set by setting the environment variable `CACHE_TTL`.
    pub cache_ttl: u64,
}

impl Config {
    const DB_POOL_MAX_SIZE: &str = "DB_POOL_MAX_SIZE";

    /// Returns the configuration of the server.
    /// The configuration is read from the environment.
    ///
    /// # Panics
    /// This function will panic if there are configuration errors during initialization.
    /// This is intentional as the application cannot start without valid configuration.
    pub fn get() -> &'static Config {
        CONFIG.get_or_init(|| {
            Self::init().unwrap_or_else(|e| {
                error!("Configuration initialization failed: {}", e);
                panic!("Failed to initialize configuration: {}", e);
            })
        })
    }

    /// Returns the HTTP binding configuration of the server.
    pub fn get_http_bind(&self) -> (String, u16) {
        info!(
            "HTTP server is listening on: {}:{}",
            self.http_bind.bold(),
            self.http_port.to_string().bold()
        );
        (self.http_bind.clone(), self.http_port)
    }

    /// Returns the HTTPS binding configuration of the server.
    pub fn get_https_bind(&self) -> (String, u16) {
        info!(
            "HTTPS server is listening on: {}:{}",
            self.https_bind.bold(),
            self.https_port.to_string().bold()
        );

        (self.https_bind.clone(), self.https_port)
    }

    /// Returns the rate limiter configuration taken from the environment.
    pub fn get_rate_limiter_config(&self) -> (u64, u64) {
        info!(
            "HTTP/S Server rate limiter max. requests {} in {} seconds.",
            self.http_rl_max_requests.to_string().bold(),
            self.http_rl_interval.to_string().bold()
        );

        (self.http_rl_max_requests, self.http_rl_interval)
    }

    /// Returns the database configuration required by the tiberius client.
    pub fn get_db_config(&self) -> TiberiusConfig {
        self.get_db_config_for_user(&self.db_user, &self.db_password)
    }

    /// Returns the database configuration required by the tiberius client.
    pub fn get_db_config_for_user(&self, user: &String, password: &String) -> TiberiusConfig {
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

    /// Initializes the configuration by reading variables from the environment.
    fn init() -> Result<Self, ConfigError> {
        dotenv().ok();
        env_logger::init();

        info!(
            "Build: time '{}', commit '{}', head_ref '{}', ",
            built_info::BUILT_TIME_UTC.bold(),
            built_info::GIT_COMMIT_HASH.unwrap_or_default().bold(),
            built_info::GIT_HEAD_REF.unwrap_or_default().bold()
        );

        // read http config with improved error handling
        let http_bind = env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0".to_string());
        let http_port: u16 = Self::parse_env_var("HTTP_PORT", "8080")?;
        let http_app_content_path = env::var("HTTP_APP_CONTENT_PATH").unwrap_or_else(|_| "./static/dist".to_owned());
        info!(
            "Serving static application content from path: {}",
            http_app_content_path.bold()
        );

        // read https config with improved error handling
        let https_bind = env::var("HTTPS_BIND").unwrap_or_else(|_| "0.0.0.0".to_string());
        let https_port: u16 = Self::parse_env_var("HTTPS_PORT", "8443")?;
        let https_cert_path = env::var("HTTPS_CERT_PATH").unwrap_or_else(|_| "./ssl/cert.pem".to_string());
        let https_key_path = env::var("HTTPS_KEY_PATH").unwrap_or_else(|_| "./ssl/key.pem".to_string());

        // read ratelimiter config with improved error handling
        let http_rl_max_requests: u64 = Self::parse_env_var("HTTP_RL_MAX_REQUESTS", "500")?;
        let http_rl_interval: u64 = Self::parse_env_var("HTTP_RL_INTERVAL", "600")?;

        // handle HTTP_WORKERS with proper error handling
        let http_workers: Option<usize> = Self::parse_optional_env_var("HTTP_WORKERS")?;

        // read db config - these are required with improved error handling
        let db_host = Self::get_required_env_var("DB_HOST")?;
        let db_port: u16 = Self::parse_env_var("DB_PORT", "1433")?;
        let db_name = Self::get_required_env_var("DB_NAME")?;
        let db_user = Self::get_required_env_var("DB_USER")?;
        let db_password = Self::get_required_env_var("DB_PASSWORD")?;
        let db_encryption: bool = Self::parse_env_var("DB_ENCRYPTION", "false")?;
        let db_pool_max_size: u32 = Self::parse_env_var("DB_POOL_MAX_SIZE", "100")?;
        let db_pool_min_idle: u32 = Self::parse_env_var("DB_POOL_MIN_IDLE", "30")?;

        // Validate database configuration values
        Self::validate_db_config(&db_host, db_port, db_pool_max_size, db_pool_min_idle)?;

        info!(
            "Database configuration: host={}, port={}, encryption={}, name={}, user={}, pool_max_size={}, pool_min_idle={}",
            db_host.bold(),
            db_port.to_string().bold(),
            db_encryption.to_string().bold(),
            db_name.bold(),
            db_user.bold(),
            db_pool_max_size.to_string().bold(),
            db_pool_min_idle.to_string().bold(),
        );

        // handle ACTIVE_REGATTA_ID with proper error handling
        let active_regatta_id: Option<i32> = Self::parse_optional_env_var("ACTIVE_REGATTA_ID")?;

        // handle cache TTL with proper error handling
        let cache_ttl: u64 = Self::parse_env_var("CACHE_TTL", "30")?;

        // Validate cache TTL
        Self::validate_cache_ttl(cache_ttl)?;

        info!(
            "Aquarius: active_regatta_id={}, cache_ttl={}s",
            active_regatta_id.unwrap_or_default().to_string().bold(),
            cache_ttl.to_string().bold()
        );

        Ok(Config {
            http_bind,
            http_port,
            https_bind,
            https_port,
            https_cert_path,
            https_key_path,
            http_rl_max_requests,
            http_rl_interval,
            http_workers,
            db_host,
            db_port,
            db_name,
            db_user,
            db_password,
            db_encryption,
            db_pool_max_size,
            db_pool_min_idle,
            active_regatta_id,
            cache_ttl,
            http_app_content_path,
        })
    }

    // Private helper methods

    /// Validates database configuration values
    fn validate_db_config(host: &str, port: u16, pool_max_size: u32, pool_min_idle: u32) -> Result<(), ConfigError> {
        // Validate host is not empty
        if host.trim().is_empty() {
            return Err(ConfigError::InvalidValue {
                var_name: "DB_HOST".to_string(),
                reason: "Database host cannot be empty".to_string(),
            });
        }

        // Validate port range
        if port == 0 {
            return Err(ConfigError::InvalidValue {
                var_name: "DB_PORT".to_string(),
                reason: "Database port cannot be 0".to_string(),
            });
        }

        // Validate pool configuration
        if pool_max_size == 0 {
            return Err(ConfigError::InvalidValue {
                var_name: Self::DB_POOL_MAX_SIZE.to_string(),
                reason: "Database pool max size must be greater than 0".to_string(),
            });
        }

        if pool_min_idle > pool_max_size {
            return Err(ConfigError::InvalidValue {
                var_name: "DB_POOL_MIN_IDLE".to_string(),
                reason: format!(
                    "Database pool min idle ({}) cannot be greater than max size ({})",
                    pool_min_idle, pool_max_size
                ),
            });
        }

        Ok(())
    }

    /// Validates cache TTL value
    fn validate_cache_ttl(ttl: u64) -> Result<(), ConfigError> {
        if ttl == 0 {
            return Err(ConfigError::InvalidValue {
                var_name: "CACHE_TTL".to_string(),
                reason: "Cache TTL must be greater than 0 seconds".to_string(),
            });
        }

        if ttl > 3600 {
            return Err(ConfigError::InvalidValue {
                var_name: "CACHE_TTL".to_string(),
                reason: format!(
                    "Cache TTL ({} seconds) is very high, maximum recommended is 3600 seconds (1 hour)",
                    ttl
                ),
            });
        }

        Ok(())
    }

    /// Helper function to parse environment variable with proper error handling
    fn parse_env_var<T>(var_name: &str, default: &str) -> Result<T, ConfigError>
    where
        T: FromStr,
        T::Err: Display,
    {
        let value = env::var(var_name).unwrap_or_else(|_| default.to_string());
        value.parse().map_err(|e: T::Err| ConfigError::ParseError {
            var_name: var_name.to_string(),
            value: value.clone(),
            error: e.to_string(),
        })
    }

    /// Helper function to get required environment variable
    fn get_required_env_var(var_name: &str) -> Result<String, ConfigError> {
        env::var(var_name).map_err(|_| ConfigError::MissingRequired(var_name.to_string()))
    }

    /// Helper function to parse optional environment variable with better error handling
    fn parse_optional_env_var<T>(var_name: &str) -> Result<Option<T>, ConfigError>
    where
        T: FromStr,
        T::Err: Display,
    {
        match env::var(var_name) {
            Ok(value) => {
                let parsed = value.parse().map_err(|e: T::Err| ConfigError::ParseError {
                    var_name: var_name.to_string(),
                    value: value.clone(),
                    error: e.to_string(),
                })?;
                Ok(Some(parsed))
            }
            Err(_) => Ok(None),
        }
    }
}
