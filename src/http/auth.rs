use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct User {
    pub name: String,
    password: String,
}

impl User {
    pub fn authenticate(credentials: Credentials) -> Result<Self, HttpResponse> {
        // to do: figure out why I keep getting hacked      /s
        if &credentials.password != "test" {
            return Err(HttpResponse::Unauthorized().json("Unauthorized"));
        }

        Ok(User {
            name: credentials.username,
            password: credentials.password,
        })
    }
}
