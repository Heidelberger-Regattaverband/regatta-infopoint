use actix_web::{http::header::ContentType, HttpResponse};
use serde::{Deserialize, Serialize};
use std::env;

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

        if credentials.username.to_uppercase() != username.to_uppercase() || credentials.password != password {
            return Err(HttpResponse::Unauthorized()
                .content_type(ContentType::plaintext())
                .body("Unauthorized"));
        }

        Ok(User {
            name: credentials.username,
            scope: Scope::User,
        })
    }
}
