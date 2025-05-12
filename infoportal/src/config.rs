use crate::built_info;
use colored::Colorize;
use dotenv::dotenv;
use log::info;
use std::{env, sync::OnceLock};
use tiberius::{AuthMethod, Config as TiberiusConfig, EncryptionLevel};

static CONFIG: OnceLock<Config> = OnceLock::new();

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
    /// Returns the configuration of the server.
    /// The configuration is read from the environment.
    pub fn get() -> &'static Config {
        CONFIG.get_or_init(Self::init)
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
    fn init() -> Self {
        dotenv().ok();
        env_logger::init();

        info!(
            "Build: time '{}', commit '{}', head_ref '{}', ",
            built_info::BUILT_TIME_UTC.bold(),
            built_info::GIT_COMMIT_HASH.unwrap_or_default().bold(),
            built_info::GIT_HEAD_REF.unwrap_or_default().bold()
        );

        // read http config
        let http_bind = env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0".to_string());
        let http_port: u16 = env::var("HTTP_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap();
        let http_app_content_path = env::var("HTTP_APP_CONTENT_PATH").unwrap_or_else(|_| "./static/dist".to_owned());

        // read https config
        let https_bind = env::var("HTTPS_BIND").unwrap_or_else(|_| "0.0.0.0".to_string());
        let https_port: u16 = env::var("HTTPS_PORT")
            .unwrap_or_else(|_| "8443".to_string())
            .parse()
            .unwrap();
        let https_cert_path = env::var("HTTPS_CERT_PATH").unwrap_or_else(|_| "./ssl/cert.pem".to_string());
        let https_key_path = env::var("HTTPS_KEY_PATH").unwrap_or_else(|_| "./ssl/key.pem".to_string());

        // read ratelimiter config
        let http_rl_max_requests: u64 = env::var("HTTP_RL_MAX_REQUESTS")
            .unwrap_or_else(|_| "500".to_string())
            .parse()
            .unwrap();
        let http_rl_interval: u64 = env::var("HTTP_RL_INTERVAL")
            .unwrap_or_else(|_| "600".to_string())
            .parse()
            .unwrap();

        let http_workers: Option<usize> = match env::var("HTTP_WORKERS") {
            // parses the value and panics if it's not a number
            Ok(workers) => Some(workers.parse().unwrap()),
            Err(_error) => Option::None,
        };

        // read db config
        let db_host = env::var("DB_HOST").expect("env variable `DB_HOST` should be set");
        let db_port: u16 = env::var("DB_PORT")
            .unwrap_or_else(|_| "1433".to_string())
            .parse()
            .unwrap();
        let db_name = env::var("DB_NAME").expect("env variable `DB_NAME` should be set");
        let db_user = env::var("DB_USER").expect("env variable `DB_USER` should be set");
        let db_password = env::var("DB_PASSWORD").expect("env variable `DB_PASSWORD` should be set");
        let db_encryption: bool = env::var("DB_ENCRYPTION")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap();
        let db_pool_max_size: u32 = env::var("DB_POOL_MAX_SIZE")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap();
        let db_pool_min_idle: u32 = env::var("DB_POOL_MIN_IDLE")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap();
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

        let active_regatta_id: Option<i32> = match env::var("ACTIVE_REGATTA_ID") {
            Ok(id) => id.parse().ok(),
            Err(_) => None,
        };
        let cache_ttl: u64 = env::var("CACHE_TTL")
            .unwrap_or_else(|_| "40".to_string())
            .parse()
            .unwrap();
        info!(
            "Aquarius: active_regatta_id={}, cache_ttl={}s",
            active_regatta_id.unwrap_or_default().to_string().bold(),
            cache_ttl.to_string().bold()
        );

        Config {
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
        }
    }
}
