mod aquarius_db;
mod db;
mod rest_api;

use crate::db::TiberiusConnectionManager;
use actix_files::Files;
use actix_web::{web::Data, App, HttpServer};
use bb8::Pool;
use log::info;
use std::{env, io::Result};

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting infopoint");

    let data = create_app_data().await;

    HttpServer::new(move || {
        App::new()
            .app_data(Data::clone(&data))
            .service(rest_api::regattas)
            .service(rest_api::heats)
            .service(rest_api::heat_registrations)
            .service(Files::new("/", "./static").show_files_listing())
            .service(Files::new("/ui", "./static/ui").index_file("index.html"))
    })
    .bind(get_http_bind())?
    .workers(4)
    .run()
    .await
}

fn get_http_bind() -> (String, u16) {
    let port = env::var("HTTP_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap();
    let host = env::var("HTTP_BIND").unwrap_or_else(|_| "127.0.0.1".to_string());
    info!("HTTP server is listening on: {host}:{port}");

    (host, port)
}

async fn create_app_data() -> Data<Pool<TiberiusConnectionManager>> {
    let pool = db::create_pool().await;
    Data::new(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_index_get() {
        let app_data = create_app_data().await;

        let app = test::init_service(
            App::new()
                .service(rest_api::regattas)
                .app_data(Data::clone(&app_data)),
        )
        .await;
        let request = test::TestRequest::get().uri("/api/regattas").to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
    }
}
