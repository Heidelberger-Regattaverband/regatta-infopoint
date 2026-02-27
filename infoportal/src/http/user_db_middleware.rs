use crate::auth::Credentials;
use crate::db::UserPoolManager;
use ::actix_web::error::ErrorInternalServerError;
use ::db::tiberius::TiberiusPool;
use ::std::sync::Arc;

/// Helper to get a database pool for the authenticated user
///
/// This function is available for future use when user-specific database credentials
/// are implemented in the authentication system.
///
/// # Arguments
/// * `user_db_credentials` - Optional user database credentials
/// * `pool_manager` - The user pool manager
/// * `default_pool` - The default application pool to use if no user credentials
///
/// # Returns
/// The user-specific pool or the default pool
#[allow(dead_code)]
pub async fn get_user_pool(
    user_db_credentials: Option<&Credentials>,
    pool_manager: &UserPoolManager,
    default_pool: Arc<TiberiusPool>,
) -> Result<Arc<TiberiusPool>, actix_web::Error> {
    // If user has custom database credentials, get their pool
    if let Some(credentials) = user_db_credentials {
        pool_manager
            .get_pool(credentials.clone())
            .await
            .map_err(|e| ErrorInternalServerError(format!("Failed to get user database pool: {}", e)))
    } else {
        // Use default application pool
        Ok(default_pool)
    }
}

/// Clean up user's database pool on logout
///
/// This function is available for future use when user-specific database credentials
/// are implemented in the authentication system.
///
/// # Arguments
/// * `user_db_credentials` - Optional user database credentials
/// * `pool_manager` - The user pool manager
#[allow(dead_code)]
pub async fn cleanup_user_pool(user_db_credentials: Option<&Credentials>, pool_manager: &UserPoolManager) {
    if let Some(credentials) = user_db_credentials {
        pool_manager.remove_pool(credentials).await;
    }
}
