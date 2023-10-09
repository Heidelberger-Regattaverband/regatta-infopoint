use crate::config::Config;
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use tiberius::{AuthMethod, Client};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

#[derive(Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    #[default]
    Guest,
    User,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub username: String,
    scope: Scope,
}

impl User {
    pub fn new(username: String, scope: Scope) -> Self {
        User { username, scope }
    }

    pub fn new_guest() -> Self {
        User {
            username: String::from("anonymous"),
            scope: Scope::Guest,
        }
    }

    pub async fn authenticate(mut credentials: Credentials) -> Result<Self, HttpResponse> {
        credentials.username = credentials.username.trim().to_owned();

        // get database config ...
        let mut db_cfg = Config::get().get_db_config();
        // ... and authenticate with given credentials
        db_cfg.authentication(AuthMethod::sql_server(
            credentials.username.clone(),
            credentials.password,
        ));

        // then try to open a connection to the MS-SQL server ...
        let tcp = TcpStream::connect(db_cfg.get_addr()).await.unwrap();
        // ... and connect with credentials
        if let Ok(client) = Client::connect(db_cfg, tcp.compat_write()).await {
            let _ = client.close().await;
            Ok(User {
                username: credentials.username,
                scope: Scope::User,
            })
        } else {
            Err(HttpResponse::Unauthorized().json(User::new_guest()))
        }
    }
}
