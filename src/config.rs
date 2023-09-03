use colored::Colorize;
use dotenv::dotenv;
use log::info;
use std::{env, sync::OnceLock};
use tiberius::{AuthMethod, EncryptionLevel};

static CONFIG: OnceLock<Config> = OnceLock::new();

pub struct Config {
    pub http_bind: String,
    pub http_port: u16,
    pub https_bind: String,
    pub https_port: u16,
    pub http_rl_max_requests: u64,
    pub http_rl_interval: u64,
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
    pub db_encryption: bool,
    pub db_pool_size: u32,
}

impl Config {
    pub fn get() -> &'static Config {
        CONFIG.get_or_init(Self::_init)
    }

    pub fn get_http_bind(&self) -> (String, u16) {
        info!(
            "HTTP server is listening on: {}:{}",
            self.http_bind.bold(),
            self.http_port.to_string().bold()
        );
        (self.http_bind.clone(), self.http_port)
    }

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

    pub fn get_db_config(&self) -> tiberius::Config {
        info!(
            "Database configuration: host={}, port={}, encryption={}, name={}, user={}",
            self.db_host.bold(),
            self.db_port.to_string().bold(),
            self.db_encryption.to_string().bold(),
            self.db_name.bold(),
            self.db_user.bold()
        );

        let mut config = tiberius::Config::new();
        config.host(self.db_host.clone());
        config.port(self.db_port);
        config.database(self.db_name.clone());
        config.authentication(AuthMethod::sql_server(self.db_user.clone(), self.db_password.clone()));
        if self.db_encryption {
            config.encryption(EncryptionLevel::Required);
            config.trust_cert();
        } else {
            config.encryption(EncryptionLevel::NotSupported);
        }
        config
    }

    fn _init() -> Self {
        dotenv().ok();

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

        // read ratelimiter config
        let http_rl_max_requests: u64 = env::var("HTTP_RL_MAX_REQUESTS")
            .unwrap_or_else(|_| "500".to_string())
            .parse()
            .unwrap();
        let http_rl_interval: u64 = env::var("HTTP_RL_INTERVAL")
            .unwrap_or_else(|_| "600".to_string())
            .parse()
            .unwrap();

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

        let db_pool_size: u32 = env::var("DB_POOL_MAX_SIZE")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap();

        Config {
            http_bind,
            http_port,
            https_bind,
            https_port,
            http_rl_max_requests,
            http_rl_interval,
            db_host,
            db_port,
            db_name,
            db_user,
            db_password,
            db_encryption,
            db_pool_size,
        }
    }
}
