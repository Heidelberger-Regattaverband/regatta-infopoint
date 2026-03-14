use crate::error::DbError;
use crate::tiberius::TiberiusConnectionManager;
use ::bb8::Pool;
use ::bb8::PooledConnection;
use ::bb8::State;
use ::std::sync::OnceLock;
use ::tiberius::Config;
use ::tokio::sync::Mutex;

/// A global instance of the Tiberius connection pool.
static POOL: OnceLock<TiberiusPool> = OnceLock::new();
/// A mutex to ensure that the global Tiberius pool is initialized only once.
static POOL_INITIALIZED: OnceLock<Mutex<bool>> = OnceLock::new();

#[derive(Debug)]
/// A pool of Tiberius connections. It is a wrapper around a `bb8::Pool` of `TiberiusConnectionManager`s.
pub struct TiberiusPool {
    /// The inner pool, which is a `bb8::Pool` of `TiberiusConnectionManager`s.
    inner: Pool<TiberiusConnectionManager>,
}

impl TiberiusPool {
    /// Returns the global instance of the `TiberiusPool`.
    /// # Returns
    /// A reference to the global `TiberiusPool`.
    /// # Panics
    /// Panics if the `TiberiusPool` has not been initialized.
    pub fn instance() -> &'static TiberiusPool {
        POOL.get().expect("TiberiusPool should be set")
    }

    /// Initializes the global `TiberiusPool`.
    ///
    /// # Arguments
    /// * `config` - The configuration for the Tiberius connection manager.
    /// * `max_size` - The maximum size of the pool.
    /// * `min_idle` - The minimum number of idle connections in the pool.
    pub async fn init(config: Config, max_size: u32, min_idle: u32) {
        if POOL.get().is_none() {
            let init_mutex = POOL_INITIALIZED.get_or_init(|| Mutex::new(false));
            let mut initialized = init_mutex.lock().await;
            if !*initialized {
                let pool = TiberiusPool::new(config, max_size, min_idle).await;
                POOL.set(pool).expect("TiberiusPool shouldn't be set");
                *initialized = true;
            }
        }
    }

    /// Creates a new `TiberiusPool` with the given configuration, maximum size, and minimum idle connections.
    ///
    /// # Arguments
    /// * `config` - The configuration for the Tiberius connection manager.
    /// * `max_size` - The maximum size of the pool.
    /// * `min_idle` - The minimum number of idle connections in the pool.
    /// # Returns
    /// A new instance of `TiberiusPool`.
    pub async fn new(config: Config, max_size: u32, min_idle: u32) -> Self {
        let manager = TiberiusConnectionManager::new(config);

        let inner = Pool::builder()
            .max_size(max_size)
            .min_idle(Some(min_idle))
            .build(manager)
            .await
            .expect("Failed to create Tiberius connection pool");
        TiberiusPool { inner }
    }

    /// Returns a connection from the global `TiberiusPool`. The connection is automatically returned to the pool when it goes out of scope.
    ///
    /// # Returns
    /// A `PooledConnection` to the Tiberius database.
    pub async fn get(&self) -> Result<PooledConnection<'_, TiberiusConnectionManager>, DbError> {
        self.inner.get().await.map_err(DbError::from)
    }

    /// Returns the current state of the global `TiberiusPool`.
    ///
    /// # Returns
    /// The current state of the pool.
    pub fn state(&self) -> State {
        self.inner.state()
    }
}
