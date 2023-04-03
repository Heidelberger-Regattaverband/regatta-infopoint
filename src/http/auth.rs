use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use std::env;

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

    pub fn authenticate(mut credentials: Credentials) -> Result<Self, HttpResponse> {
        let username = env::var("AUTH_USER_NAME").unwrap();
        let password = env::var("AUTH_USER_PASSWORD").unwrap();

        credentials.username = credentials.username.trim().to_owned();

        if credentials.username.to_uppercase() != username.to_uppercase() || credentials.password != password {
            return Err(HttpResponse::Unauthorized().json(User::new_guest()));
        }

        Ok(User {
            username: credentials.username,
            scope: Scope::User,
        })
    }
}
