use crate::error::DbError;
use crate::tiberius::TiberiusPool;
use ::std::collections::HashMap;
use ::std::sync::Arc;
use ::tiberius::AuthMethod;
use ::tiberius::Config as TiberiusConfig;
use ::tokio::sync::RwLock;

/// Manager for per-user database connection pools
pub struct UserPoolManager {
    /// Cache of connection pools by user credentials
    pools: RwLock<HashMap<String, Arc<TiberiusPool>>>,

    config: TiberiusConfig,
}

impl UserPoolManager {
    /// Create a new UserPoolManager with base database configuration
    pub fn new(config: TiberiusConfig) -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
            config,
        }
    }

    pub async fn get_pool(&self, username: &str) -> Option<Arc<TiberiusPool>> {
        let pools = self.pools.read().await;
        pools.get(username).cloned()
    }

    /// Get or create a connection pool for the given user credentials
    pub async fn create_pool(&self, username: &str, password: &str) -> Result<Arc<TiberiusPool>, DbError> {
        // First check if pool exists (read lock)
        if let Some(pool) = self.get_pool(username).await {
            return Ok(pool);
        }

        // Pool doesn't exist, create it (write lock)
        let mut pools = self.pools.write().await;

        // Double-check in case another task created it while we were waiting
        if let Some(pool) = pools.get(username) {
            return Ok(pool.clone());
        }

        // Create new pool with user-specific credentials
        let mut config = self.config.clone();
        config.authentication(AuthMethod::sql_server(username, password));

        let pool = Arc::new(TiberiusPool::new(config, 5, 1).await);
        pools.insert(username.to_string(), pool.clone());
        Ok(pool)
    }

    /// Remove a user's connection pool (e.g., on logout)
    #[allow(dead_code)]
    pub async fn remove_pool(&self, username: &String) {
        let mut pools = self.pools.write().await;
        pools.remove(username);
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
