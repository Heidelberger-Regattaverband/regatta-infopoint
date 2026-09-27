use ::actix_session::storage::LoadError;
use ::actix_session::storage::SaveError;
use ::actix_session::storage::SessionKey;
use ::actix_session::storage::SessionStore;
use ::actix_session::storage::UpdateError;
use ::actix_web::cookie::time::Duration;
use ::chrono::DateTime;
use ::chrono::Utc;
use ::db::error::DbError;
use ::db::tiberius::TiberiusPool;
use ::db::tiberius::user_pool::UserPoolManager;
use ::serde_json;
use ::std::collections::HashMap;
use ::std::sync::Arc;
use ::tiberius::Query;
use ::tracing::debug;
use ::tracing::error;
use ::uuid::Uuid;

/// The session state key used by actix-identity 0.9 to store the logged-in username.
/// See `actix_identity::config::Configuration::id_key` default value.
const IDENTITY_KEY: &str = "actix_identity.user_id";

/// A server-side session store backed by MS-SQL.
///
/// Stores session data in a `Sessions` table, which is created at server startup
/// if it does not already exist. On session deletion (explicit logout or TTL cleanup),
/// the corresponding user pool is released via [`UserPoolManager`].
#[derive(Clone)]
pub struct DbSessionStore {
    pool: Arc<TiberiusPool>,
    user_pool_manager: Arc<UserPoolManager>,
}

impl DbSessionStore {
    pub fn new(pool: Arc<TiberiusPool>, user_pool_manager: Arc<UserPoolManager>) -> Self {
        Self {
            pool,
            user_pool_manager,
        }
    }

    /// Creates the `Sessions` table and its expiry index if they do not already exist.
    pub async fn ensure_table(&self) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;
        conn.simple_query(
            "IF OBJECT_ID('Sessions', 'U') IS NULL \
             BEGIN \
               CREATE TABLE Sessions ( \
                 session_key  NVARCHAR(128) NOT NULL, \
                 session_data NVARCHAR(MAX) NOT NULL, \
                 expires_at   DATETIME2     NOT NULL, \
                 username     NVARCHAR(256) NULL, \
                 CONSTRAINT PK_Sessions PRIMARY KEY (session_key) \
               ); \
               CREATE INDEX IX_Sessions_ExpiresAt ON Sessions (expires_at); \
             END",
        )
        .await?
        .into_row()
        .await?;
        Ok(())
    }

    /// Deletes all expired sessions and releases the corresponding user pools.
    ///
    /// Should be called periodically from a background task.
    pub async fn cleanup_expired(&self) {
        let conn = match self.pool.get().await {
            Ok(c) => c,
            Err(e) => {
                error!(%e, "Session cleanup: failed to get DB connection");
                return;
            }
        };
        let mut conn = conn;
        let query = Query::new("DELETE FROM Sessions OUTPUT DELETED.username WHERE expires_at < GETUTCDATE()");
        match query.query(&mut conn).await {
            Ok(stream) => match stream.into_results().await {
                Ok(results) => {
                    let mut removed = 0u64;
                    for rows in results {
                        for row in rows {
                            removed += 1;
                            if let Ok(Some(username)) = row.try_get::<&str, _>(0) {
                                debug!(username, "Session expired, removing user pool:");
                                self.user_pool_manager.remove_pool(username).await;
                            }
                        }
                    }
                    if removed > 0 {
                        debug!(removed, "Cleaned up expired sessions:");
                    }
                }
                Err(e) => error!(%e, "Session cleanup: failed to read deleted rows"),
            },
            Err(e) => error!(%e, "Session cleanup: DELETE query failed"),
        }
    }
}

type SessionState = HashMap<String, String>;

impl SessionStore for DbSessionStore {
    async fn load(&self, session_key: &SessionKey) -> Result<Option<SessionState>, LoadError> {
        let mut conn = self.pool.get().await.map_err(|e: DbError| LoadError::Other(e.into()))?;
        let mut query =
            Query::new("SELECT session_data FROM Sessions WHERE session_key = @P1 AND expires_at > GETUTCDATE()");
        query.bind(session_key.as_ref());
        let row = query
            .query(&mut conn)
            .await
            .map_err(|e: ::tiberius::error::Error| LoadError::Other(e.into()))?
            .into_row()
            .await
            .map_err(|e: ::tiberius::error::Error| LoadError::Other(e.into()))?;

        match row {
            None => Ok(None),
            Some(row) => {
                let data: &str = row
                    .get(0)
                    .ok_or_else(|| LoadError::Other(DbError::Custom("missing session_data column".into()).into()))?;
                let state: SessionState =
                    serde_json::from_str(data).map_err(|e| LoadError::Deserialization(e.into()))?;
                Ok(Some(state))
            }
        }
    }

    async fn save(&self, session_state: SessionState, ttl: &Duration) -> Result<SessionKey, SaveError> {
        let key = Uuid::new_v4().to_string();
        let data = serde_json::to_string(&session_state).map_err(|e| SaveError::Serialization(e.into()))?;
        let expires_at = expiry(ttl);
        let username = extract_username(&session_state);
        let username = username.as_deref();

        let mut conn = self.pool.get().await.map_err(|e: DbError| SaveError::Other(e.into()))?;
        let mut query = Query::new(
            "INSERT INTO Sessions (session_key, session_data, expires_at, username) \
             VALUES (@P1, @P2, @P3, @P4)",
        );
        query.bind(key.as_str());
        query.bind(data.as_str());
        query.bind(expires_at);
        query.bind(username);
        query
            .execute(&mut conn)
            .await
            .map_err(|e: ::tiberius::error::Error| SaveError::Other(e.into()))?;

        SessionKey::try_from(key).map_err(|e| SaveError::Other(DbError::Custom(e.to_string()).into()))
    }

    async fn update(
        &self,
        session_key: SessionKey,
        session_state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, UpdateError> {
        let data = serde_json::to_string(&session_state).map_err(|e| UpdateError::Serialization(e.into()))?;
        let expires_at = expiry(ttl);
        let username = extract_username(&session_state);
        let username = username.as_deref();

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e: DbError| UpdateError::Other(e.into()))?;
        let mut query = Query::new(
            "UPDATE Sessions SET session_data = @P1, expires_at = @P2, username = @P3 \
             WHERE session_key = @P4",
        );
        query.bind(data.as_str());
        query.bind(expires_at);
        query.bind(username);
        query.bind(session_key.as_ref());
        let result = query
            .execute(&mut conn)
            .await
            .map_err(|e: ::tiberius::error::Error| UpdateError::Other(e.into()))?;

        // Row expired between load and update — insert as new row.
        if result.rows_affected().iter().sum::<u64>() == 0 {
            let mut conn = self
                .pool
                .get()
                .await
                .map_err(|e: DbError| UpdateError::Other(e.into()))?;
            let mut insert = Query::new(
                "INSERT INTO Sessions (session_key, session_data, expires_at, username) \
                 VALUES (@P1, @P2, @P3, @P4)",
            );
            insert.bind(session_key.as_ref());
            insert.bind(data.as_str());
            insert.bind(expires_at);
            insert.bind(username);
            insert
                .execute(&mut conn)
                .await
                .map_err(|e: ::tiberius::error::Error| UpdateError::Other(e.into()))?;
        }

        Ok(session_key)
    }

    async fn update_ttl(&self, session_key: &SessionKey, ttl: &Duration) -> Result<(), anyhow::Error> {
        let expires_at = expiry(ttl);
        let mut conn = self.pool.get().await?;
        let mut query = Query::new("UPDATE Sessions SET expires_at = @P1 WHERE session_key = @P2");
        query.bind(expires_at);
        query.bind(session_key.as_ref());
        query.execute(&mut conn).await?;
        Ok(())
    }

    async fn delete(&self, session_key: &SessionKey) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        let mut query = Query::new("DELETE FROM Sessions OUTPUT DELETED.username WHERE session_key = @P1");
        query.bind(session_key.as_ref());
        let results = query.query(&mut conn).await?.into_results().await?;
        for rows in results {
            for row in rows {
                if let Ok(Some(username)) = row.try_get::<&str, _>(0) {
                    debug!(username, "Session deleted, removing user pool:");
                    self.user_pool_manager.remove_pool(username).await;
                }
            }
        }
        Ok(())
    }
}

/// Compute the absolute expiry `DateTime<Utc>` from a relative TTL.
fn expiry(ttl: &Duration) -> DateTime<Utc> {
    Utc::now() + chrono::Duration::seconds(ttl.whole_seconds())
}

/// Extract the plain username string from the session state.
///
/// actix-session stores all values JSON-encoded, so the raw map entry for the
/// username is `"sa"` (with surrounding quotes). `serde_json::from_str` removes them.
fn extract_username(session_state: &SessionState) -> Option<String> {
    session_state
        .get(IDENTITY_KEY)
        .and_then(|v| serde_json::from_str::<String>(v).ok())
}
