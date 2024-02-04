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
    dev::{Service, ServiceFactory, ServiceRequest, ServiceResponse},
    web::{self, scope, Data},
    App, Error, HttpServer,
};
use actix_web_prometheus::{PrometheusMetrics, PrometheusMetricsBuilder};
use colored::Colorize;
use futures::FutureExt;
use log::{debug, info, warn};
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use rustls_pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use std::{
    fs::File,
    future::Ready,
    io::{self, BufReader},
    path::Path,
    sync::{Arc, Mutex},
    time::{self, Instant},
};

/// Path to REST API
pub const PATH_REST_API: &str = "/api";
/// Path to Infoportal UI
const INFOPORTAL: &str = "infoportal";

/// The server struct contains the configuration of the server.
pub struct Server<'a> {
    /// The configuration of the server.
    config: &'a Config,
}

/// The server implementation.
impl<'a> Server<'a> {
    /// Creates s new server instance with given configuration.
    /// # Arguments
    /// * `config` - The configuration of the server.
    /// # Returns
    /// * `Server` - The server.
    pub fn new(config: &'a Config) -> Server {
        Server { config }
    }

    /// Starts the server.
    /// # Returns
    /// * `io::Result<()>` - The result of the server start.
    /// # Panics
    /// If the server can't be started.
    pub async fn start(&self) -> io::Result<()> {
        let start = Instant::now();

        let aquarius = create_app_data().await;
        let (rl_max_requests, rl_interval) = self.config.get_rate_limiter_config();
        let secret_key = Self::get_secret_key();
        let http_app_content_path = self.config.http_app_content_path.clone();

        let worker_count = Arc::new(Mutex::new(0));
        let prometheus = Self::get_prometeus();

        let factory_closure = move || {
            let mut current_count = worker_count.lock().unwrap();
            *current_count += 1;
            debug!("Created new HTTP worker: count={}", current_count.to_string().bold());

            // get app with some middlewares initialized
            Self::get_app(secret_key.clone(), rl_max_requests, rl_interval)
                // collect metrics about requests and responses
                .wrap(prometheus.clone())
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
                        .service(rest_api::calculate_scoring)
                        .service(rest_api::get_statistics)
                        .service(rest_api::login)
                        .service(rest_api::identity)
                        .service(rest_api::logout),
                )
                .service(
                    Files::new(INFOPORTAL, http_app_content_path.clone())
                        .index_file("index.html")
                        .use_last_modified(true)
                        .use_etag(true)
                        .redirect_to_slash_directory(),
                )
                // redirect from / to /infoportal
                .service(web::redirect("/", INFOPORTAL))
                .service(rest_api::monitor)
        };

        let mut http_server = HttpServer::new(factory_closure)
            // always bind to http
            .bind(self.config.get_http_bind())?;

        // also bind to https if config is available
        if let Some(rustls_cfg) = Self::get_rustls_config() {
            let https_bind = self.config.get_https_bind();
            http_server = http_server.bind_rustls_021(https_bind, rustls_cfg)?;
        }

        // configure number of workers if env. variable is set
        if let Some(workers) = self.config.http_workers {
            http_server = http_server.workers(workers);
        }

        // finally run http server
        let server = http_server.run();
        info!("Infoportal started in {:?}", start.elapsed());
        server.await
    }

    /// Returns a new App instance with some middlewares initialized.
    /// # Arguments
    /// * `secret_key` - The secret key used to encrypt the session cookie.
    /// * `rl_max_requests` - The maximum number of requests in the given interval.
    /// * `rl_interval` - The interval in seconds.
    /// # Returns
    /// * `App` - The app.
    fn get_app(
        secret_key: Key,
        rl_max_requests: u64,
        rl_interval: u64,
    ) -> App<
        impl ServiceFactory<
            ServiceRequest,
            Config = (),
            Response = ServiceResponse<EitherBody<BoxBody>>,
            Error = Error,
            InitError = (),
        >,
    > {
        App::new()
            // Install the identity framework first.
            .wrap(IdentityMiddleware::default())
            // adds support for HTTPS sessions
            .wrap(Self::get_session_middleware(secret_key))
            // adds support for rate limiting of HTTP requests
            .wrap(Self::get_rate_limiter(rl_max_requests, rl_interval))
            .wrap_fn(|req, srv| {
                // println!("Hi from start. You requested: {}", req.path());
                srv.call(req).map(|res| {
                    // println!("Hi from response");
                    res
                })
            })
    }

    /// Returns a new SessionMiddleware instance.
    /// # Arguments
    /// * `secret_key` - The secret key used to encrypt the session cookie.
    /// # Returns
    /// * `SessionMiddleware` - The session middleware.
    /// # Panics
    /// If the session middleware can't be created.
    fn get_session_middleware(secret_key: Key) -> SessionMiddleware<CookieSessionStore> {
        const SECS_OF_WEEKEND: i64 = 60 * 60 * 24 * 2;
        SessionMiddleware::builder(CookieSessionStore::default(), secret_key)
            .cookie_secure(true)
            .cookie_http_only(true)
            // allow the cookie only from the current domain
            .cookie_same_site(SameSite::Strict)
            .session_lifecycle(PersistentSession::default().session_ttl(Duration::seconds(SECS_OF_WEEKEND)))
            .cookie_path("".to_string())
            .build()
    }

    /// Returns a new secret key instance.
    /// # Returns
    /// * `Key` - The secret key.
    /// # Panics
    /// If the secret key can't be created.
    fn get_secret_key() -> Key {
        Key::generate()
    }

    /// Returns a new PrometheusMetrics instance.
    /// # Returns
    /// * `PrometheusMetrics` - The prometheus metrics.
    /// # Panics
    /// If the prometheus metrics can't be created.
    fn get_prometeus() -> Arc<PrometheusMetrics> {
        Arc::new(
            PrometheusMetricsBuilder::new("api")
                .endpoint("/metrics")
                .build()
                .unwrap(),
        )
    }

    /// Returns a new RateLimiter instance.
    /// # Arguments
    /// * `max_requests` - The maximum number of requests in the given interval.
    /// * `interval` - The interval in seconds.
    /// # Returns
    /// * `RateLimiter` - The rate limiter.
    /// # Panics
    /// If the rate limiter can't be created.
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

    /// Returns HTTPS server configuration if available.
    /// # Returns
    /// * `Option<ServerConfig>` - The HTTPS server configuration.
    /// # Panics
    /// If the HTTPS server configuration can't be created.
    /// # Remarks
    /// The HTTPS server configuration is only created if the certificate and private key files are available.
    /// The certificate and private key files are configured in the environment.
    fn get_rustls_config() -> Option<ServerConfig> {
        let cert_pem_path = Path::new(&Config::get().https_cert_path);
        let key_pem_path = Path::new(&Config::get().https_key_path);

        info!(
            "Current working directory is {}",
            std::env::current_dir().unwrap().display().to_string().bold()
        );

        if cert_pem_path.exists() && cert_pem_path.is_file() && key_pem_path.exists() && key_pem_path.is_file() {
            // load TLS key/cert files
            info!(
                "Try to load TLS config from: certificate {} and private key {}.",
                &Config::get().https_cert_path.bold(),
                &Config::get().https_key_path.bold()
            );

            if let (Ok(cert_file), Ok(key_file)) = (File::open(cert_pem_path), File::open(key_pem_path)) {
                let cert_reader = &mut BufReader::new(cert_file);
                let cert_chain = certs(cert_reader).map(|cert| cert.unwrap()).collect();

                let key_reader = &mut BufReader::new(key_file);
                // convert files to key/cert objects
                let mut keys: Vec<PrivatePkcs8KeyDer> =
                    pkcs8_private_keys(key_reader).map(|cert| cert.unwrap()).collect();

                // no keys could be parsedpter for each variant
                if keys.is_empty() {
                    warn!("Could not locate PKCS 8 private keys.");
                    return None;
                }

                // init server config builder with safe defaults
                let config = ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(cert_chain, PrivateKeyDer::Pkcs8(keys.remove(0)))
                    .unwrap();
                Some(config)
            } else {
                warn!(
                    "Can't open one or both files: certificate {} and private key {}.",
                    &Config::get().https_cert_path.bold(),
                    &Config::get().https_key_path.bold()
                );
                None
            }
        } else {
            warn!(
                "One or both are not existing or are directories: certificate {} and private key {}.",
                &Config::get().https_cert_path.bold(),
                &Config::get().https_key_path.bold()
            );
            None
        }
    }
}

pub async fn create_app_data() -> Data<Aquarius> {
    Data::new(Aquarius::new().await)
}
