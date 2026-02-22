use crate::auth::Credentials;
use crate::config::CONFIG;
use ::db::bb8::Pool;
use ::db::error::DbError;
use ::db::tiberius::TiberiusConnectionManager;
use ::db::tiberius::TiberiusPool;
use ::std::collections::HashMap;
use ::std::sync::Arc;
use ::tokio::sync::RwLock;

/// Manager for per-user database connection pools
pub struct UserPoolManager {
    /// Cache of connection pools by user credentials
    pools: Arc<RwLock<HashMap<Credentials, Arc<TiberiusPool>>>>,
}

impl UserPoolManager {
    /// Create a new UserPoolManager with base database configuration
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a connection pool for the given user credentials
    pub async fn get_pool(&self, credentials: Credentials) -> Result<Arc<TiberiusPool>, DbError> {
        // First check if pool exists (read lock)
        {
            let pools = self.pools.read().await;
            if let Some(pool) = pools.get(&credentials) {
                return Ok(Arc::clone(pool));
            }
        }

        // Pool doesn't exist, create it (write lock)
        let mut pools = self.pools.write().await;

        // Double-check in case another task created it while we were waiting
        if let Some(pool) = pools.get(&credentials) {
            return Ok(Arc::clone(pool));
        }

        // Create new pool with user-specific credentials
        let config = CONFIG.get_db_config_for_user(&credentials.username, credentials.password.value());

        let manager = TiberiusConnectionManager::new(config);

        let inner = Pool::builder()
            .max_size(CONFIG.db_pool_max_size)
            .min_idle(Some(CONFIG.db_pool_min_idle))
            .build(manager)
            .await?;

        let pool = Arc::new(TiberiusPool::from_pool(inner));
        pools.insert(credentials.clone(), Arc::clone(&pool));

        Ok(pool)
    }

    /// Remove a user's connection pool (e.g., on logout)
    pub async fn remove_pool(&self, credentials: &Credentials) {
        let mut pools = self.pools.write().await;
        pools.remove(credentials);
    }

    /// Clear all connection pools
    #[allow(dead_code)]
    pub async fn clear_all(&self) {
        let mut pools = self.pools.write().await;
        pools.clear();
    }

    /// Get the number of active connection pools
    #[allow(dead_code)]
    pub async fn pool_count(&self) -> usize {
        let pools = self.pools.read().await;
        pools.len()
    }
}
