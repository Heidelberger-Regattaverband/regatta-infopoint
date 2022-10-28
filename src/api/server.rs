use crate::{api::rest_api, db::aquarius::Aquarius};
use actix_extensible_rate_limit::{
    backend::{memory::InMemoryBackend, SimpleInputFunctionBuilder},
    RateLimiter,
};
use actix_files::Files;
use actix_web::{
    web::{scope, Data},
    App, HttpServer,
};
use actix_web_prometheus::PrometheusMetricsBuilder;
use log::info;
use std::{env, io::Result, time::Duration};

pub static SCOPE_API: &str = "/api";

pub struct Server {}

impl Server {
    pub async fn start() -> Result<()> {
        let prometheus = PrometheusMetricsBuilder::new("api")
            .endpoint("/metrics")
            .build()
            .unwrap();

        let data = create_app_data().await;
        // A backend is responsible for storing rate limit data, and choosing whether to allow/deny requests

        let http_rate_limiter = get_http_rate_limiter();

        let mut http_server = HttpServer::new(move || {
            let input = SimpleInputFunctionBuilder::new(
                Duration::from_secs(http_rate_limiter.1),
                http_rate_limiter.0,
            )
            .real_ip_key()
            .build();
            let rate_limiter = RateLimiter::builder(InMemoryBackend::builder().build(), input)
                .add_headers()
                .build();

            App::new()
                .wrap(rate_limiter)
                .wrap(prometheus.clone())
                .app_data(Data::clone(&data))
                .service(
                    scope(SCOPE_API)
                        .service(rest_api::get_regattas)
                        .service(rest_api::get_regatta)
                        .service(rest_api::get_races)
                        .service(rest_api::get_heats)
                        .service(rest_api::get_registrations)
                        .service(rest_api::get_heat_registrations)
                        .service(rest_api::get_scoring),
                )
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

        info!("Starting Infoportal");
        // finally run http server
        http_server.run().await
    }
}

fn get_http_bind() -> (String, u16) {
    let port = env::var("HTTP_PORT")
        .expect("env variable `HTTP_PORT` should be set")
        .parse()
        .unwrap();
    let host = env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0".to_string());
    info!("HTTP server is listening on: {host}:{port}");

    (host, port)
}

fn get_http_workers() -> Option<usize> {
    match env::var("HTTP_WORKERS") {
        // parses the value and panics if it's not a number
        Ok(workers) => Some(workers.parse().unwrap()),
        Err(_error) => Option::None,
    }
}

fn get_http_rate_limiter() -> (u64, u64) {
    let max_requests = env::var("HTTP_RL_MAX_REQUESTS")
        .expect("env variable `HTTP_RL_MAX_REQUESTS` should be set")
        .parse()
        .unwrap();
    let interval = env::var("HTTP_RL_INTERVAL")
        .expect("env variable `HTTP_RL_INTERVAL` should be set")
        .parse()
        .unwrap();
    log::debug!(
        "HTTP Server rate limiter max. requests {} in {} seconds.",
        max_requests,
        interval
    );
    (max_requests, interval)
}

pub async fn create_app_data() -> Data<Aquarius> {
    Data::new(Aquarius::new().await)
}
