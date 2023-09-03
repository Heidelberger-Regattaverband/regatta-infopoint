use crate::config::{self, Config};
use async_trait::async_trait;
use bb8::{ManageConnection, Pool, PooledConnection, State};
use colored::Colorize;
use log::debug;
use std::sync::{Arc, Mutex};
use tiberius::{error::Error, Client, Config as TiberiusConfig};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

#[derive(Debug)]
pub struct TiberiusConnectionManager {
    config: TiberiusConfig,
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

    fn create_config() -> TiberiusConfig {
        let config = Config::get().get_db_config();
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
    type Connection = Client<Compat<TcpStream>>;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let tcp = TcpStream::connect(self.config.get_addr()).await?;
        tcp.set_nodelay(true)?;
        let result = Client::connect(self.config.clone(), tcp.compat_write()).await;
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
        let db_pool_size: u32 = config::Config::get().db_pool_size;

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
