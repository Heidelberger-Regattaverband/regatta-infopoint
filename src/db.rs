//! Tiberius support for the `bb8` connection pool.
//!
//! # Example
//!
//! ```
//! use futures::future::join_all;
//!
//! #[tokio::main]
//! async fn main() {
//!     let manager = TiberiusConnectionManager::new(Config::from_ado_string(SERVICE_CONFIG.get_connection_string()).unwrap()).unwrap();
//!     let pool = bb8::Pool::builder().build(manager).await.unwrap();
//!
//!     let mut handles = vec![];
//!
//!     for _i in 0..10 {
//!         let pool = pool.clone();
//!
//!         handles.push(tokio::spawn(async move {
//!             let mut conn = pool.get().await.unwrap();
//!
//!             let stream = client.simple_query("SELECT @@VERSION").await?;
//!             let rows: Vec<Row> = stream.into_first_result().await?;
//!         }));
//!     }
//!
//!     join_all(handles).await;
//! }
//! ```
use async_std::net::TcpStream;
use async_trait::async_trait;
use bb8::Pool;
use log::info;
use std::env;
use tiberius::Config;

// exposes sub-modules
pub mod aquarius;
pub mod utils;

pub type TiberiusPool = Pool<TiberiusConnectionManager>;

#[derive(Clone, Debug)]
pub struct TiberiusConnectionManager {
    config: Config,
}

impl TiberiusConnectionManager {
    /// Create a new `TiberiusConnectionManager`.
    fn new(config: Config) -> tiberius::Result<TiberiusConnectionManager> {
        Ok(TiberiusConnectionManager { config })
    }
}

#[async_trait]
impl bb8::ManageConnection for TiberiusConnectionManager {
    type Connection = tiberius::Client<TcpStream>;
    type Error = tiberius::error::Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let tcp = TcpStream::connect(&self.config.get_addr()).await?;
        tcp.set_nodelay(true)?;
        tiberius::Client::connect(self.config.clone(), tcp).await
    }

    async fn is_valid(&self, connection: &mut Self::Connection) -> Result<(), Self::Error> {
        //debug!("Checking {:?}", conn);
        connection.simple_query("").await?.into_row().await?;
        Ok(())
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}

pub async fn create_pool() -> Pool<TiberiusConnectionManager> {
    let config = create_config();
    let manager = TiberiusConnectionManager::new(config).unwrap();
    Pool::builder().max_size(20).build(manager).await.unwrap()
}

fn get_db_port() -> u16 {
    env::var("DB_PORT")
        .unwrap_or_else(|_| "1433".to_string())
        .parse()
        .unwrap()
}

fn get_db_host() -> String {
    env::var("DB_HOST").unwrap_or_else(|_| "8e835d.online-server.cloud".to_string())
}

fn get_db_name() -> String {
    env::var("DB_NAME").unwrap_or_else(|_| "Regatta_2022_Test".to_string())
}

fn get_db_user() -> String {
    env::var("DB_USER").unwrap_or_else(|_| "sa".to_string())
}

fn get_db_password() -> String {
    env::var("DB_PASSWORD").unwrap_or_default()
}

fn create_config() -> tiberius::Config {
    let db_host = get_db_host();
    let db_port = get_db_port();
    let db_name = get_db_name();
    let db_user = get_db_user();

    info!(
        "Database configuration: host={}, port={}, name={}, user={}",
        db_host, db_port, db_name, db_user
    );

    let mut config = tiberius::Config::new();
    config.host(db_host);
    config.port(db_port);
    config.database(db_name);
    config.authentication(tiberius::AuthMethod::sql_server(db_user, get_db_password()));
    config.encryption(tiberius::EncryptionLevel::NotSupported);
    config
}
