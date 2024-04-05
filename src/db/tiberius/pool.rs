use crate::config::Config;
use async_trait::async_trait;
use bb8::{ManageConnection, Pool, PooledConnection, State};
use colored::Colorize;
use log::debug;
use std::sync::{Arc, Mutex};
use tiberius::{error::Error, Client, Config as TiberiusConfig};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

/// A connection manager for Tiberius connections.
#[derive(Debug)]
pub struct TiberiusConnectionManager {
    /// The database configuration.
    config: TiberiusConfig,

    /// The number of created connections.
    count: Arc<Mutex<u32>>,
}

impl TiberiusConnectionManager {
    /// Creates a new `TiberiusConnectionManager`.
    fn new() -> TiberiusConnectionManager {
        TiberiusConnectionManager {
            config: Config::get().get_db_config(),
            count: Arc::new(Mutex::new(0)),
        }
    }

    /// Increments the connection count.
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

/// A pool of Tiberius connections.
pub(crate) struct TiberiusPool {
    /// The inner pool.
    inner: Pool<TiberiusConnectionManager>,

    /// The number of created connections.
    count: Arc<Mutex<u32>>,
}

impl TiberiusPool {
    /// Creates a new `TiberiusPool`.
    pub(crate) async fn new() -> Self {
        let manager = TiberiusConnectionManager::new();
        let count = manager.count.clone();

        let inner = Pool::builder()
            .max_size(Config::get().db_pool_max_size)
            .min_idle(Some(Config::get().db_pool_min_idle))
            .build(manager)
            .await
            .unwrap();
        TiberiusPool { inner, count }
    }

    /// Returns a connection from the pool. The connection is automatically returned to the pool when it goes out of scope.
    pub(crate) async fn get(&self) -> PooledConnection<'_, TiberiusConnectionManager> {
        self.inner.get().await.unwrap()
    }

    /// Returns the current state of the pool.
    pub(crate) fn state(&self) -> State {
        self.inner.state()
    }

    /// Returns the number of created connections.
    pub(crate) fn created(&self) -> u32 {
        *self.count.lock().unwrap()
    }
}
