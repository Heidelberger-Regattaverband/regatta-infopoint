use std::env;

use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
enum Scope {
    #[default]
    Guest,
    User,
}

#[derive(Serialize)]
pub struct User {
    pub name: String,
    scope: Scope,
}

impl User {
    pub fn authenticate(credentials: Credentials) -> Result<Self, HttpResponse> {
        let username = env::var("USER_NAME").unwrap();
        let password = env::var("USER_PASSWORD").unwrap();

        if &credentials.username != &username || &credentials.password != &password {
            return Err(HttpResponse::Unauthorized().json("Unauthorized"));
        }

        Ok(User {
            name: credentials.username,
            scope: Scope::User,
        })
    }
}
