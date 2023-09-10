use crate::{config::Config, db::aquarius::Aquarius, http::rest_api};
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
use log::{debug, info, warn};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::{
    fs::File,
    future::Ready,
    io::{self, BufReader},
    time::{self, Instant},
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

    pub async fn start(config: &Config) -> io::Result<()> {
        let start = Instant::now();

        let aquarius = create_app_data().await;
        let (rl_max_requests, rl_interval) = config.get_rate_limiter_config();
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
                        .service(rest_api::get_filters)
                        .service(rest_api::get_heat)
                        .service(rest_api::get_kiosk)
                        .service(rest_api::get_registrations)
                        .service(rest_api::calculate_scoring)
                        .service(rest_api::get_statistics)
                        .service(rest_api::login)
                        .service(rest_api::identity)
                        .service(rest_api::logout),
                )
                .service(
                    Files::new(INFOPORTAL, "./static/webapp".to_owned())
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
        .bind(config.get_http_bind())?;

        // bind https if config is available
        if let Some(rustls_cfg) = Self::get_rustls_config() {
            let https_bind = config.get_https_bind();
            http_server = http_server.bind_rustls_021(https_bind, rustls_cfg)?;
        }

        // configure number of workers if env. variable is set
        if let Some(workers) = config.http_workers {
            http_server = http_server.workers(workers);
        }

        // finally run http server
        let server = http_server.run();
        info!("Starting Infoportal in {:?}", start.elapsed());
        server.await
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

    fn get_rustls_config() -> Option<ServerConfig> {
        // init server config builder with safe defaults
        let config = ServerConfig::builder().with_safe_defaults().with_no_client_auth();

        let cert_pem_path = Config::get().https_cert_path.clone();
        let key_pem_path = Config::get().https_key_path.clone();

        debug!(
            "Current working directory is {}",
            std::env::current_dir().unwrap().display().to_string().bold()
        );

        // load TLS key/cert files
        debug!(
            "Try to load TLS config from: certificate {} and private key {}.",
            cert_pem_path.bold(),
            key_pem_path.bold()
        );

        match File::open(cert_pem_path) {
            Ok(cert_file) => {
                let cert_reader = &mut BufReader::new(cert_file);
                let cert_chain: Vec<Certificate> = certs(cert_reader).unwrap().into_iter().map(Certificate).collect();

                match File::open(key_pem_path) {
                    Ok(key_file) => {
                        let key_reader = &mut BufReader::new(key_file);
                        // convert files to key/cert objects
                        let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_reader)
                            .unwrap()
                            .into_iter()
                            .map(PrivateKey)
                            .collect();

                        // no keys could be parsedpter for each variant
                        if keys.is_empty() {
                            warn!("Could not locate PKCS 8 private keys.");
                            return None;
                        }

                        Some(config.with_single_cert(cert_chain, keys.remove(0)).unwrap())
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }
}

pub async fn create_app_data() -> Data<Aquarius> {
    Data::new(Aquarius::new().await)
}
