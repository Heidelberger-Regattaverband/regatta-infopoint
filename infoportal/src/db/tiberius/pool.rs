use crate::config::Config;
use aquarius::db::tiberius::TiberiusConnectionManager;
use bb8::{Pool, PooledConnection, State};
use std::sync::OnceLock;

static POOL: OnceLock<TiberiusPool> = OnceLock::new();

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
        let manager = TiberiusConnectionManager::new(Config::get().get_db_config());

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
