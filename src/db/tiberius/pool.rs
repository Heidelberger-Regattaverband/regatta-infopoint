use async_std::net::TcpStream;
use async_trait::async_trait;
use bb8::{ManageConnection, Pool, PooledConnection, State};
use colored::Colorize;
use log::{debug, info};
use std::{
    env,
    sync::{Arc, Mutex},
};
use tiberius::{error::Error, AuthMethod, Client, Config, EncryptionLevel};

#[derive(Debug)]
pub struct TiberiusConnectionManager {
    config: Config,
    count: Arc<Mutex<u32>>,
}

impl TiberiusConnectionManager {
    /// Creates a new `TiberiusConnectionManager`.
    fn new() -> TiberiusConnectionManager {
        TiberiusConnectionManager {
            config: Self::create_config(),
            count: Arc::new(Mutex::new(0)),
        }
    }

    fn create_config() -> tiberius::Config {
        let db_host = env::var("DB_HOST").expect("env variable `DB_HOST` should be set");
        let db_port: u16 = env::var("DB_PORT")
            .expect("env variable `DB_PORT` should be set")
            .parse()
            .unwrap();
        let db_name = env::var("DB_NAME").expect("env variable `DB_NAME` should be set");
        let db_user = env::var("DB_USER").expect("env variable `DB_USER` should be set");
        let db_password = env::var("DB_PASSWORD").expect("env variable `DB_PASSWORD` should be set");
        let db_encryption: bool = env::var("DB_ENCRYPTION")
            .expect("env variable `DB_ENCRYPTION` should be set")
            .parse()
            .unwrap();
        info!(
            "Database configuration: host={}, port={}, encryption={}, name={}, user={}",
            db_host.bold(),
            db_port.to_string().bold(),
            db_encryption.to_string().bold(),
            db_name.bold(),
            db_user.bold()
        );

        let mut config = Config::new();
        config.host(db_host);
        config.port(db_port);
        config.database(db_name);
        config.authentication(AuthMethod::sql_server(db_user, db_password));
        if db_encryption {
            config.encryption(EncryptionLevel::Required);
        } else {
            config.encryption(EncryptionLevel::NotSupported);
        }
        config.trust_cert();
        config
    }

    fn inc_count(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
        debug!("Created new DB connection: count={}", count.to_string().bold());
    }
}

#[async_trait]
impl ManageConnection for TiberiusConnectionManager {
    type Connection = Client<TcpStream>;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let tcp = TcpStream::connect(self.config.get_addr()).await?;
        tcp.set_nodelay(true)?;
        let result = Client::connect(self.config.clone(), tcp).await;
        self.inc_count();
        result
    }

    async fn is_valid(&self, connection: &mut Self::Connection) -> Result<(), Self::Error> {
        //debug!("Checking {:?}", conn);
        connection.simple_query("").await?.into_row().await?;
        Ok(())
    }

    fn has_broken(&self, _connection: &mut Self::Connection) -> bool {
        false
    }
}

pub struct TiberiusPool {
    inner: Pool<TiberiusConnectionManager>,
    count: Arc<Mutex<u32>>,
}

impl TiberiusPool {
    pub async fn new() -> Self {
        let db_pool_size: u32 = env::var("DB_POOL_MAX_SIZE")
            .expect("env variable `DB_POOL_MAX_SIZE` should be set")
            .parse()
            .unwrap();

        let manager = TiberiusConnectionManager::new();
        let count = manager.count.clone();

        debug!(
            "Creating DB pool with configuration: max_size={}",
            db_pool_size.to_string().bold()
        );

        TiberiusPool {
            inner: Pool::builder().max_size(db_pool_size).build(manager).await.unwrap(),
            count,
        }
    }

    pub async fn get(&self) -> PooledConnection<'_, TiberiusConnectionManager> {
        self.inner.get().await.unwrap()
    }

    pub fn state(&self) -> State {
        self.inner.state()
    }

    pub fn created(&self) -> u32 {
        *self.count.lock().unwrap()
    }
}
