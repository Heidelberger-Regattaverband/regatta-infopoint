use bb8::{Pool, PooledConnection, State};
use std::sync::OnceLock;
use tiberius::Config as TiberiusConfig;

use super::TiberiusConnectionManager;

static POOL: OnceLock<TiberiusPool> = OnceLock::new();

#[derive(Debug)]
/// A pool of Tiberius connections.
pub struct TiberiusPool {
    /// The inner pool.
    inner: Pool<TiberiusConnectionManager>,
}

impl TiberiusPool {
    /// Returns the current instance of the `TiberiusPool`.
    pub fn instance() -> &'static TiberiusPool {
        POOL.get().expect("TiberiusPool not set")
    }

    /// Initializes the `TiberiusPool`.
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
    pub async fn get(&self) -> PooledConnection<'_, TiberiusConnectionManager> {
        self.inner.get().await.unwrap()
    }

    /// Returns the current state of the pool.
    pub fn state(&self) -> State {
        self.inner.state()
    }
}
