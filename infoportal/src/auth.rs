use crate::config::CONFIG;
use ::actix_web::HttpResponse;
use ::serde::{Deserialize, Serialize};
use ::tiberius::Client;
use ::tokio::net::TcpStream;
use ::tokio_util::compat::TokioAsyncWriteCompatExt;
use ::tracing::warn;
use ::utoipa::ToSchema;

/// The credentials struct contains the username and the password of the user.
/// The credentials are used to authenticate the user.
#[derive(Clone, Hash, PartialEq, Eq, Deserialize, ToSchema)]
pub struct Credentials {
    /// The username of the user.
    pub username: String,
    /// The password of the user.
    pub password: String,
}

/// The scope enum contains the possible scopes of the user.
/// The scope is used to determine the permissions of the user.
#[derive(Serialize, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Scope {
    /// The user is a guest.
    #[default]
    Guest,
    /// The user is authenticated.
    User,
    /// The user is an administrator.
    Admin,
}

/// The user struct contains the username and the scope of the user.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct User {
    /// The username of the user.
    pub(crate) username: String,
    /// The scope of the user.
    pub(crate) scope: Scope,
}

impl User {
    /// Creates a new user with the given username and scope.
    /// # Arguments
    /// * `username` - The username of the user.
    /// * `scope` - The scope of the user.
    /// # Returns
    /// * `User` - The user.
    pub fn new(username: String, scope: Scope) -> Self {
        User { username, scope }
    }

    /// Creates a new user with scope guest.
    /// # Returns
    /// * `User` - The guest user.
    pub fn new_guest() -> Self {
        User {
            username: String::from("anonymous"),
            scope: Scope::Guest,
        }
    }

    /// Authenticates the user with the given credentials.
    /// # Arguments
    /// * `credentials` - The credentials of the user.
    /// # Returns
    /// * `Ok(User)` - The authenticated user.
    /// * `Err(HttpResponse)` - The error response.
    pub async fn authenticate(credentials: Credentials) -> Result<Self, HttpResponse> {
        let mut username: String = Default::default();
        credentials.username.trim().clone_into(&mut username);

        // get database config with given credentials
        let db_cfg = CONFIG.get_db_config_for_user(&username, &credentials.password);

        // then try to open a connection to the MS-SQL server ...
        let tcp = match TcpStream::connect(db_cfg.get_addr()).await {
            Ok(stream) => stream,
            Err(err) => {
                warn!(?err, "Failed to connect to database");
                return Err(HttpResponse::Unauthorized().json(User::new_guest()));
            }
        };

        // ... and connect with credentials
        if let Ok(client) = Client::connect(db_cfg, tcp.compat_write()).await {
            let _ = client.close().await;
            let scope: Scope = if &credentials.username == "sa" {
                Scope::Admin
            } else {
                Scope::User
            };
            Ok(User {
                username: credentials.username,
                scope,
            })
        } else {
            Err(HttpResponse::Unauthorized().json(User::new_guest()))
        }
    }
}
