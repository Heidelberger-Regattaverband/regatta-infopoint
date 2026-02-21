use crate::auth::Credentials;
use ::db::bb8::Pool;
use ::db::error::DbError;
use ::db::tiberius::TiberiusConnectionManager;
use ::db::tiberius::TiberiusPool;
use ::std::collections::HashMap;
use ::std::sync::Arc;
use ::tiberius::AuthMethod;
use ::tiberius::Config as TiberiusConfig;
use ::tiberius::EncryptionLevel;
use ::tokio::sync::RwLock;

/// Configuration for database connection
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub encryption: bool,
    pub pool_max_size: u32,
    pub pool_min_idle: u32,
}

impl DbConfig {
    /// Create TiberiusConfig for specific user credentials
    pub fn to_tiberius_config(&self, username: &str, password: &str) -> TiberiusConfig {
        let mut config = TiberiusConfig::new();
        config.host(&self.host);
        config.port(self.port);
        config.database(&self.database);
        config.authentication(AuthMethod::sql_server(username, password));
        if self.encryption {
            config.encryption(EncryptionLevel::Required);
            config.trust_cert();
        } else {
            config.encryption(EncryptionLevel::NotSupported);
        }
        config
    }
}

/// Manager for per-user database connection pools
pub struct UserPoolManager {
    /// Cache of connection pools by user credentials
    pools: Arc<RwLock<HashMap<Credentials, Arc<TiberiusPool>>>>,
    /// Base database configuration (host, database name, etc.)
    base_config: DbConfig,
}

impl UserPoolManager {
    /// Create a new UserPoolManager with base database configuration
    pub fn new(base_config: DbConfig) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            base_config,
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
        let config = self
            .base_config
            .to_tiberius_config(&credentials.username, &credentials.password);

        let manager = TiberiusConnectionManager::new(config);

        let inner = Pool::builder()
            .max_size(self.base_config.pool_max_size)
            .min_idle(Some(self.base_config.pool_min_idle))
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
