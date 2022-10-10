mod db;
mod rest_api;

use actix_extensible_rate_limit::{
    backend::{memory::InMemoryBackend, SimpleInputFunctionBuilder},
    RateLimiter,
};
use actix_files::Files;
use actix_web::{web::Data, App, HttpServer};
use actix_web_prometheus::PrometheusMetricsBuilder;
use db::aquarius::Aquarius;
use log::info;
use std::{env, io::Result, time::Duration};

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Infoportal");

    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .unwrap();

    let data = create_app_data().await;
    // A backend is responsible for storing rate limit data, and choosing whether to allow/deny requests

    let mut http_server = HttpServer::new(move || {
        let input = SimpleInputFunctionBuilder::new(Duration::from_secs(600), 200)
            .real_ip_key()
            .build();
        let rate_limiter = RateLimiter::builder(InMemoryBackend::builder().build(), input)
            .add_headers()
            .build();

        App::new()
            .wrap(rate_limiter)
            .wrap(prometheus.clone())
            .app_data(Data::clone(&data))
            .service(rest_api::get_regattas)
            .service(rest_api::get_regatta)
            .service(rest_api::get_heats)
            .service(rest_api::get_heat_registrations)
            .service(rest_api::get_scoring)
            .service(
                Files::new("/infoportal", "./static/infopoint")
                    .index_file("index.html")
                    .use_last_modified(true)
                    .use_etag(true)
                    .redirect_to_slash_directory(),
            )
    })
    .bind(get_http_bind())?;

    // configure number of workers if env. variable is set
    let workers = get_http_workers();
    if workers.is_some() {
        http_server = http_server.workers(workers.unwrap());
    }

    // finally run http server
    http_server.run().await
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

fn get_http_workers() -> Option<usize> {
    match env::var("HTTP_WORKERS") {
        // parses the value and panics if it's not a number
        Ok(workers) => Some(workers.parse().unwrap()),
        Err(_error) => Option::None,
    }
}

async fn create_app_data() -> Data<Aquarius> {
    Data::new(Aquarius::new().await)
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
