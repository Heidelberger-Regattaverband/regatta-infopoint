use crate::{db::aquarius::Aquarius, http::rest_api};
use actix_extensible_rate_limit::{
    backend::{memory::InMemoryBackend, SimpleInput, SimpleInputFunctionBuilder, SimpleOutput},
    RateLimiter,
};
use actix_files::Files;
use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    body::{BoxBody, EitherBody},
    cookie::{time::Duration, Key, SameSite},
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web::{self, scope, Data},
    App, Error, HttpServer,
};
use actix_web_prometheus::{PrometheusMetrics, PrometheusMetricsBuilder, StreamMetrics};
use colored::Colorize;
use log::{debug, info};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::{
    env,
    fs::File,
    future::Ready,
    io::{self, BufReader},
    time,
};

/// Path to REST API
pub const PATH_REST_API: &str = "/api";
/// Path to Infoportal UI
const INFOPORTAL: &str = "infoportal";

pub struct Server {}

impl Server {
    fn get_app(
        secret_key: Key,
        rl_max_requests: u64,
        rl_interval: u64,
    ) -> App<
        impl ServiceFactory<
            ServiceRequest,
            Config = (),
            Response = ServiceResponse<EitherBody<StreamMetrics<BoxBody>>>,
            Error = Error,
            InitError = (),
        >,
    > {
        App::new()
            // Install the identity framework first.
            .wrap(IdentityMiddleware::default())
            // adds support for HTTPS sessions
            .wrap(Self::get_session_middleware(secret_key))
            // collect metrics about requests and responses
            .wrap(Self::get_prometeus())
            // adds support for rate limiting of HTTP requests
            .wrap(Self::get_rate_limiter(rl_max_requests, rl_interval))
    }

    pub async fn start() -> io::Result<()> {
        let aquarius = create_app_data().await;
        let rustls_cfg = Self::get_rustls_config();
        let (rl_max_requests, rl_interval) = Self::get_rate_limiter_config();
        let http_bind = Self::get_http_bind();
        let https_bind = Self::get_https_bind();
        let secret_key = Self::get_secret_key();

        let mut http_server = HttpServer::new(move || {
            // get app with some middlewares initialized
            Self::get_app(secret_key.clone(), rl_max_requests, rl_interval)
                .app_data(aquarius.clone())
                .service(
                    scope(PATH_REST_API)
                        .service(rest_api::get_club)
                        .service(rest_api::get_regattas)
                        .service(rest_api::get_club_registrations)
                        .service(rest_api::get_participating_clubs)
                        .service(rest_api::get_active_regatta)
                        .service(rest_api::get_regatta)
                        .service(rest_api::get_race)
                        .service(rest_api::get_races)
                        .service(rest_api::get_heats)
                        .service(rest_api::get_kiosk)
                        .service(rest_api::get_registrations)
                        .service(rest_api::get_heat_registrations)
                        .service(rest_api::get_scoring)
                        .service(rest_api::get_statistics)
                        .service(rest_api::login)
                        .service(rest_api::identity)
                        .service(rest_api::logout),
                )
                .service(
                    Files::new(INFOPORTAL, "./static/".to_owned() + INFOPORTAL)
                        .index_file("index.html")
                        .use_last_modified(true)
                        .use_etag(true)
                        .redirect_to_slash_directory(),
                )
                // redirect from / to /infoportal
                .service(web::redirect("/", INFOPORTAL))
                .service(rest_api::monitor)
        })
        // bind http
        .bind(http_bind)?
        // bind https
        .bind_rustls(https_bind, rustls_cfg)?;

        // configure number of workers if env. variable is set
        if let Some(workers) = get_http_workers() {
            http_server = http_server.workers(workers);
        }

        info!("Starting Infoportal");
        // finally run http server
        http_server.run().await
    }

    fn get_session_middleware(secret_key: Key) -> SessionMiddleware<CookieSessionStore> {
        const SECS_OF_WEEKEND: i64 = 60 * 60 * 24 * 2;
        SessionMiddleware::builder(CookieSessionStore::default(), secret_key)
            .cookie_secure(true)
            .cookie_http_only(true)
            // allow the cookie only from the current domain
            .cookie_same_site(SameSite::Strict)
            .session_lifecycle(PersistentSession::default().session_ttl(Duration::seconds(SECS_OF_WEEKEND)))
            .cookie_path(INFOPORTAL.to_string())
            .build()
    }

    // The secret key would usually be read from a configuration file/environment variables.
    fn get_secret_key() -> Key {
        Key::generate()
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
        max_requests: u64,
        interval: u64,
    ) -> RateLimiter<InMemoryBackend, SimpleOutput, impl Fn(&ServiceRequest) -> Ready<Result<SimpleInput, Error>>> {
        let input = SimpleInputFunctionBuilder::new(time::Duration::from_secs(interval), max_requests)
            .real_ip_key()
            .build();

        RateLimiter::builder(InMemoryBackend::builder().build(), input)
            .add_headers()
            .build()
    }

    /// Returns the rate limiter configuration taken from the environment.
    fn get_rate_limiter_config() -> (u64, u64) {
        let max_requests: u64 = env::var("HTTP_RL_MAX_REQUESTS")
            .expect("env variable `HTTP_RL_MAX_REQUESTS` should be set")
            .parse()
            .unwrap();
        let interval: u64 = env::var("HTTP_RL_INTERVAL")
            .expect("env variable `HTTP_RL_INTERVAL` should be set")
            .parse()
            .unwrap();
        debug!(
            "HTTP/S Server rate limiter max. requests {} in {} seconds.",
            max_requests.to_string().bold(),
            interval.to_string().bold()
        );
        (max_requests, interval)
    }

    fn get_rustls_config() -> ServerConfig {
        // init server config builder with safe defaults
        let config = ServerConfig::builder().with_safe_defaults().with_no_client_auth();

        let cert_pem_path = env::var("HTTPS_CERT_PATH").unwrap_or_else(|_| "./ssl/cert.pem".to_string());
        let key_pem_path = env::var("HTTPS_KEY_PATH").unwrap_or_else(|_| "./ssl/key.pem".to_string());

        debug!(
            "Current working directory is {}",
            std::env::current_dir().unwrap().display().to_string().bold()
        );

        // load TLS key/cert files
        debug!(
            "Loading TLS config: certificate {} and private key {}.",
            cert_pem_path.bold(),
            key_pem_path.bold()
        );
        let cert_file = &mut BufReader::new(File::open(cert_pem_path).unwrap());
        let key_file = &mut BufReader::new(File::open(key_pem_path).unwrap());

        // convert files to key/cert objects
        let cert_chain = certs(cert_file).unwrap().into_iter().map(Certificate).collect();
        let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
            .unwrap()
            .into_iter()
            .map(PrivateKey)
            .collect();

        // exit if no keys could be parsedpter for each variant
        /////////////////
        if keys.is_empty() {
            eprintln!("Could not locate PKCS 8 private keys.");
            std::process::exit(1);
        }

        config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
    }

    fn get_http_bind() -> (String, u16) {
        let port: u16 = env::var("HTTP_PORT")
            .expect("env variable `HTTP_PORT` should be set")
            .parse()
            .unwrap();
        let host = env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0".to_string());
        debug!(
            "HTTP server is listening on: {}:{}",
            host.bold(),
            port.to_string().bold()
        );

        (host, port)
    }

    fn get_https_bind() -> (String, u16) {
        let port: u16 = env::var("HTTPS_PORT")
            .expect("env variable `HTTPS_PORT` should be set")
            .parse()
            .unwrap();
        let host = env::var("HTTPS_BIND").unwrap_or_else(|_| "0.0.0.0".to_string());
        debug!(
            "HTTPS server is listening on: {}:{}",
            host.bold(),
            port.to_string().bold()
        );

        (host, port)
    }
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
