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
    Admin,
}

#[derive(Serialize)]
pub struct User {
    pub name: String,
    scope: Scope,
}

impl User {
    pub fn authenticate(credentials: Credentials) -> Result<Self, HttpResponse> {
        if &credentials.password != "test" {
            return Err(HttpResponse::Unauthorized().json("Unauthorized"));
        }

        Ok(User {
            name: credentials.username,
            scope: Scope::User,
        })
    }
}
