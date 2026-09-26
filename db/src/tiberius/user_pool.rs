use crate::error::DbError;
use crate::tiberius::TiberiusPool;
use ::std::collections::HashMap;
use ::std::sync::Arc;
use ::tiberius::AuthMethod;
use ::tiberius::Config as TiberiusConfig;
use ::tokio::sync::RwLock;
use ::tracing::debug;

/// Manager for per-user database connection pools.
///
/// Each username maps to a shared pool and a session reference count.
/// The pool is created on first login and dropped only when the last session
/// for that user logs out, so concurrent sessions for the same user do not
/// interfere with each other.
pub struct UserPoolManager {
    /// Active pools keyed by username, with their session reference count.
    pools: RwLock<HashMap<String, (Arc<TiberiusPool>, u64)>>,

    config: TiberiusConfig,
}

impl UserPoolManager {
    /// Create a new UserPoolManager with base database configuration.
    pub fn new(config: TiberiusConfig) -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Return the pool for `username` without changing its reference count.
    pub async fn get_pool(&self, username: &str) -> Option<Arc<TiberiusPool>> {
        let pools = self.pools.read().await;
        pools.get(username).map(|(pool, _)| pool.clone())
    }

    /// Get or create a connection pool for the given user credentials.
    ///
    /// Increments the session reference count so that a concurrent logout from
    /// another session of the same user does not destroy this session's pool.
    /// Call `remove_pool` when the session ends.
    pub async fn create_pool(&self, username: &str, password: &str) -> Result<Arc<TiberiusPool>, DbError> {
        // Fast path: pool already exists — increment session count under write lock.
        {
            let mut pools = self.pools.write().await;
            if let Some((pool, count)) = pools.get_mut(username) {
                *count += 1;
                debug!(username, sessions = *count, "User pool session attached:");
                return Ok(pool.clone());
            }
        }

        // Slow path: create the pool outside the lock (`TiberiusPool::new` is async).
        let mut config = self.config.clone();
        config.authentication(AuthMethod::sql_server(username, password));
        let new_pool = Arc::new(TiberiusPool::new(config, 5, 1).await?);

        // Re-acquire write lock: a concurrent login may have inserted the pool.
        let mut pools = self.pools.write().await;
        if let Some((pool, count)) = pools.get_mut(username) {
            *count += 1;
            debug!(
                username,
                sessions = *count,
                "User pool session attached (concurrent login):"
            );
            Ok(pool.clone())
        } else {
            pools.insert(username.to_string(), (new_pool.clone(), 1));
            debug!(username, sessions = 1, "User pool created:");
            Ok(new_pool)
        }
    }

    /// Decrement the session reference count for `username`.
    ///
    /// The pool is removed only when the last session for that user logs out.
    pub async fn remove_pool(&self, username: &str) {
        let mut pools = self.pools.write().await;
        let remove = if let Some((_, count)) = pools.get_mut(username) {
            if *count > 1 {
                *count -= 1;
                debug!(
                    username,
                    sessions = *count,
                    "User pool session detached, pool still active:"
                );
                false
            } else {
                true
            }
        } else {
            false
        };
        if remove {
            pools.remove(username);
            debug!(username, sessions = 0, "User pool removed:");
        }
    }

    /// Return a snapshot of all active sessions keyed by username.
    ///
    /// Uses a non-blocking `try_read` so it is safe to call from synchronous
    /// contexts (e.g. monitoring). Returns an empty vec on the rare occasion
    /// that a write lock is held concurrently.
    pub fn try_active_sessions(&self) -> Vec<(String, u64)> {
        match self.pools.try_read() {
            Ok(guard) => guard
                .iter()
                .map(|(username, (_, count))| (username.clone(), *count))
                .collect(),
            Err(_) => vec![],
        }
    }
}
