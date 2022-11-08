use crate::{db::aquarius::Aquarius, http::rest_api};
use actix_extensible_rate_limit::{
    backend::{memory::InMemoryBackend, SimpleInput, SimpleInputFunctionBuilder, SimpleOutput},
    RateLimiter,
};
use actix_files::Files;
use actix_web::{
    dev::ServiceRequest,
    web::{scope, Data},
    App, Error, HttpServer,
};
use actix_web_lab::web as web_lab;
use actix_web_prometheus::{PrometheusMetrics, PrometheusMetricsBuilder};
use log::{debug, info};
use std::{env, future::Ready, io::Result, time::Duration};

pub static SCOPE_API: &str = "/api";
static PATH_INFOPORTAL: &str = "/infoportal/";

pub struct Server {}

impl Server {
    pub async fn start() -> Result<()> {
        let data = create_app_data().await;

        let rl_config = Self::get_rate_limiter_config();

        let mut http_server = HttpServer::new(move || {
            App::new()
                .wrap(Self::get_rate_limiter(rl_config))
                .wrap(Self::get_prometeus())
                .app_data(Data::clone(&data))
                .service(
                    scope(SCOPE_API)
                        .service(rest_api::get_regattas)
                        .service(rest_api::get_regatta)
                        .service(rest_api::get_race)
                        .service(rest_api::get_races)
                        .service(rest_api::get_heats)
                        .service(rest_api::get_registrations)
                        .service(rest_api::get_heat_registrations)
                        .service(rest_api::get_scoring),
                )
                .service(
                    Files::new(PATH_INFOPORTAL, "./static/infoportal")
                        .index_file("index.html")
                        .use_last_modified(true)
                        .use_etag(true)
                        .redirect_to_slash_directory(),
                )
                .service(web_lab::redirect("/", PATH_INFOPORTAL))
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

    /// Returns a new PrometheusMetrics instance.
    fn get_prometeus() -> PrometheusMetrics {
        PrometheusMetricsBuilder::new("api")
            .endpoint("/metrics")
            .build()
            .unwrap()
    }

    /// Returns a new RateLimiter instance.
    fn get_rate_limiter(
        rl_config: (u64, u64),
    ) -> RateLimiter<
        InMemoryBackend,
        SimpleOutput,
        impl Fn(&ServiceRequest) -> Ready<core::result::Result<SimpleInput, Error>>,
    > {
        let input = SimpleInputFunctionBuilder::new(Duration::from_secs(rl_config.1), rl_config.0)
            .real_ip_key()
            .build();

        RateLimiter::builder(InMemoryBackend::builder().build(), input)
            .add_headers()
            .build()
    }

    /// Returns the rate limiter configuration taken from the environment.
    fn get_rate_limiter_config() -> (u64, u64) {
        let max_requests = env::var("HTTP_RL_MAX_REQUESTS")
            .expect("env variable `HTTP_RL_MAX_REQUESTS` should be set")
            .parse()
            .unwrap();
        let interval = env::var("HTTP_RL_INTERVAL")
            .expect("env variable `HTTP_RL_INTERVAL` should be set")
            .parse()
            .unwrap();
        debug!(
            "HTTP Server rate limiter max. requests {} in {} seconds.",
            max_requests, interval
        );
        (max_requests, interval)
    }
}

fn get_http_bind() -> (String, u16) {
    let port = env::var("HTTP_PORT")
        .expect("env variable `HTTP_PORT` should be set")
        .parse()
        .unwrap();
    let host = env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0".to_string());
    debug!("HTTP server is listening on: {host}:{port}");

    (host, port)
}

fn get_http_workers() -> Option<usize> {
    match env::var("HTTP_WORKERS") {
        // parses the value and panics if it's not a number
        Ok(workers) => Some(workers.parse().unwrap()),
        Err(_error) => Option::None,
    }
}

pub async fn create_app_data() -> Data<Aquarius> {
    Data::new(Aquarius::new().await)
}
