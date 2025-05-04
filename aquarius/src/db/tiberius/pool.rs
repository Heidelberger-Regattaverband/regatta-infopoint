use crate::db::tiberius::TiberiusConnectionManager;
use bb8::{Pool, PooledConnection, State};
use std::sync::OnceLock;
use tiberius::Config as TiberiusConfig;

static POOL: OnceLock<TiberiusPool> = OnceLock::new();

#[derive(Debug)]
/// A pool of Tiberius connections. It is a wrapper around a `bb8::Pool` of `TiberiusConnectionManager`s.
pub struct TiberiusPool {
    /// The inner pool, which is a `bb8::Pool` of `TiberiusConnectionManager`s.
    inner: Pool<TiberiusConnectionManager>,
}

impl TiberiusPool {
    /// Returns the current instance of the `TiberiusPool`.
    pub fn instance() -> &'static TiberiusPool {
        POOL.get().expect("TiberiusPool not set")
    }

    /// Initializes the `TiberiusPool`.
    ///
    /// # Arguments
    /// * `config` - The configuration for the Tiberius connection manager.
    /// * `max_size` - The maximum size of the pool.
    /// * `min_idle` - The minimum number of idle connections in the pool.
    pub async fn init(config: TiberiusConfig, max_size: u32, min_idle: u32) {
        let manager = TiberiusConnectionManager::new(config);

        let inner = Pool::builder()
            .max_size(max_size)
            .min_idle(Some(min_idle))
            .build(manager)
            .await
            .unwrap();

        if POOL.get().is_none() {
            POOL.set(TiberiusPool { inner }).expect("TiberiusPool already set")
        }
    }

    /// Returns a connection from the pool. The connection is automatically returned to the pool when it goes out of scope.
    ///
    /// # Returns
    /// A `PooledConnection` to the Tiberius database.
    pub async fn get(&self) -> PooledConnection<'_, TiberiusConnectionManager> {
        self.inner.get().await.unwrap()
    }

    /// Returns the current state of the pool.
    ///
    /// # Returns
    /// The current state of the pool.
    pub fn state(&self) -> State {
        self.inner.state()
    }
}
