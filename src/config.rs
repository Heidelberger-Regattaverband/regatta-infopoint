use colored::Colorize;
use dotenv::dotenv;
use log::info;
use std::{env, sync::OnceLock};
use tiberius::{AuthMethod, Config as TiberiusConfig, EncryptionLevel};

static CONFIG: OnceLock<Config> = OnceLock::new();

pub struct Config {
    pub http_bind: String,
    pub http_port: u16,
    pub https_bind: String,
    pub https_port: u16,
    pub https_cert_path: String,
    pub https_key_path: String,
    pub http_rl_max_requests: u64,
    pub http_rl_interval: u64,
    pub http_workers: Option<usize>,
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
    pub db_encryption: bool,
    pub db_pool_max_size: u32,
    pub db_pool_min_idle: u32,
    pub active_regatta_id: Option<i32>,
    pub cache_ttl: u64,
}

impl Config {
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
        let mut config = TiberiusConfig::new();
        config.host(&self.db_host);
        config.port(self.db_port);
        config.database(&self.db_name);
        config.authentication(AuthMethod::sql_server(&self.db_user, &self.db_password));
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

        // read http config
        let http_bind = env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0".to_string());
        let http_port: u16 = env::var("HTTP_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap();

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
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap();
        let db_pool_max_size: u32 = env::var("DB_POOL_MAX_SIZE")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap();
        let db_pool_min_idle: u32 = env::var("DB_POOL_MIN_IDLE")
            .unwrap_or_else(|_| "10".to_string())
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

        let mut active_regatta_id: Option<i32> = None;
        if let Ok(id) = env::var("ACTIVE_REGATTA_ID") {
            active_regatta_id = Some(id.parse().unwrap());
        }
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
        }
    }
}
