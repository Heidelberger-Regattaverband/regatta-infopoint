use crate::config::Config;
use async_trait::async_trait;
use bb8::{ManageConnection, Pool, PooledConnection, State};
use std::sync::OnceLock;
use tiberius::{error::Error, Client, Config as TiberiusConfig};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

static POOL: OnceLock<TiberiusPool> = OnceLock::new();
/// A connection manager for Tiberius connections.
#[derive(Debug)]
pub struct TiberiusConnectionManager {
    /// The database configuration.
    config: TiberiusConfig,
}

impl TiberiusConnectionManager {
    /// Creates a new `TiberiusConnectionManager`.
    fn new() -> TiberiusConnectionManager {
        TiberiusConnectionManager {
            config: Config::get().get_db_config(),
        }
    }
}

#[async_trait]
impl ManageConnection for TiberiusConnectionManager {
    type Connection = Client<Compat<TcpStream>>;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let tcp = TcpStream::connect(self.config.get_addr()).await?;
        tcp.set_nodelay(true)?;
        Client::connect(self.config.clone(), tcp.compat_write()).await
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

#[derive(Debug)]
/// A pool of Tiberius connections.
pub(crate) struct TiberiusPool {
    /// The inner pool.
    inner: Pool<TiberiusConnectionManager>,
}

impl TiberiusPool {
    /// Returns the current instance of the `TiberiusPool`.
    pub(crate) fn instance() -> &'static TiberiusPool {
        POOL.get().expect("TiberiusPool not set")
    }

    /// Initializes the `TiberiusPool`.
    pub(crate) async fn init() {
        let manager = TiberiusConnectionManager::new();

        let inner = Pool::builder()
            .max_size(Config::get().db_pool_max_size)
            .min_idle(Some(Config::get().db_pool_min_idle))
            .build(manager)
            .await
            .unwrap();

        if POOL.get().is_none() {
            POOL.set(TiberiusPool { inner }).expect("TiberiusPool already set")
        }
    }

    /// Returns a connection from the pool. The connection is automatically returned to the pool when it goes out of scope.
    pub(crate) async fn get(&self) -> PooledConnection<'_, TiberiusConnectionManager> {
        self.inner.get().await.unwrap()
    }

    /// Returns the current state of the pool.
    pub(crate) fn state(&self) -> State {
        self.inner.state()
    }
}
