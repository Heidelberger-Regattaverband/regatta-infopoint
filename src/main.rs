mod db;
mod rest_api;

use crate::db::pool::{create_pool, TiberiusConnectionManager};
use actix_extensible_rate_limit::{
    backend::{memory::InMemoryBackend, SimpleInputFunctionBuilder},
    RateLimiter,
};
use actix_files::Files;
use actix_web::{web::Data, App, HttpServer};
use bb8::Pool;
use log::info;
use std::{env, io::Result, time::Duration};

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Infopoint");

    let data = create_app_data().await;
    // A backend is responsible for storing rate limit data, and choosing whether to allow/deny requests

    HttpServer::new(move || {
        // Assign a limit of 5 requests per minute per client ip address
        let input = SimpleInputFunctionBuilder::new(Duration::from_secs(60), 50)
            .real_ip_key()
            .build();
        let backend = InMemoryBackend::builder().build();
        let middleware = RateLimiter::builder(backend, input).add_headers().build();

        App::new()
            .wrap(middleware)
            .app_data(Data::clone(&data))
            .service(rest_api::get_regattas)
            .service(rest_api::get_regatta)
            .service(rest_api::get_heats)
            .service(rest_api::get_heat_registrations)
            .service(
                Files::new("/infopoint", "./static/infopoint")
                    .index_file("index.html")
                    .use_last_modified(true)
                    .use_etag(true)
                    .redirect_to_slash_directory(),
            )
    })
    .bind(get_http_bind())?
    // .workers(get_http_workers())
    .run()
    .await
}

fn get_http_bind() -> (String, u16) {
    let port = get_http_port();
    let host = env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0".to_string());
    info!("HTTP server is listening on: {host}:{port}");

    (host, port)
}

fn get_http_port() -> u16 {
    env::var("HTTP_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap()
}

fn get_http_workers() -> usize {
    env::var("HTTP_PORT")
        .unwrap_or_else(|_| "4".to_string())
        .parse()
        .unwrap()
}

async fn create_app_data() -> Data<Pool<TiberiusConnectionManager>> {
    let pool = create_pool().await;
    Data::new(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, test::TestRequest, App};

    #[actix_web::test]
    async fn test_get_regattas() {
        let app_data = create_app_data().await;

        let app = test::init_service(
            App::new()
                .service(rest_api::get_regattas)
                .app_data(Data::clone(&app_data)),
        )
        .await;
        let request = TestRequest::get().uri("/api/regattas").to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
    }

    #[actix_web::test]
    async fn test_get_heats() {
        let app_data = create_app_data().await;

        let app = test::init_service(
            App::new()
                .service(rest_api::get_heats)
                .app_data(Data::clone(&app_data)),
        )
        .await;
        let request = TestRequest::get()
            .uri("/api/regattas/12/heats")
            .to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
    }
}
