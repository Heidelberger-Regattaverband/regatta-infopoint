use crate::config::CONFIG;
use crate::http::api_doc;
use crate::http::rest_api;
use ::actix_extensible_rate_limit::RateLimiter;
use ::actix_extensible_rate_limit::backend::SimpleInput;
use ::actix_extensible_rate_limit::backend::SimpleInputFunctionBuilder;
use ::actix_extensible_rate_limit::backend::SimpleOutput;
use ::actix_extensible_rate_limit::backend::memory::InMemoryBackend;
use ::actix_files::Files;
use ::actix_identity::IdentityMiddleware;
use ::actix_identity::config::LogoutBehavior;
use ::actix_session::SessionMiddleware;
use ::actix_session::config::PersistentSession;
use ::actix_session::config::TtlExtensionPolicy;
use ::actix_session::storage::CookieSessionStore;
use ::actix_web::App;
use ::actix_web::Error;
use ::actix_web::HttpResponse;
use ::actix_web::HttpServer;
use ::actix_web::body::BoxBody;
use ::actix_web::body::EitherBody;
use ::actix_web::cookie::Key;
use ::actix_web::cookie::SameSite;
use ::actix_web::dev::Service;
use ::actix_web::dev::ServiceFactory;
use ::actix_web::dev::ServiceRequest;
use ::actix_web::dev::ServiceResponse;
use ::actix_web::http::header::HeaderName;
use ::actix_web::http::header::HeaderValue;
use ::actix_web::web::Data;
use ::actix_web::web::{self};
use ::actix_web_prom::PrometheusMetrics;
use ::actix_web_prom::PrometheusMetricsBuilder;
use ::db::aquarius::Aquarius;
use ::db::error::DbError;
use ::db::tiberius::user_pool::UserPoolManager;
use ::futures::FutureExt;
use ::futures::try_join;
use ::prometheus::Encoder;
use ::prometheus::Registry;
use ::prometheus::TextEncoder;
use ::rustls::ServerConfig;
use ::rustls_pemfile::certs;
use ::rustls_pemfile::pkcs8_private_keys;
use ::rustls_pki_types::PrivateKeyDer;
use ::rustls_pki_types::PrivatePkcs8KeyDer;
use ::std::fs::File;
use ::std::future::Ready;
use ::std::io::BufReader;
use ::std::io::Result as IoResult;
use ::std::path::Path;
use ::std::sync::Arc;
use ::std::sync::Mutex;
use ::std::time::Duration;
use ::std::time::Instant;
use ::tracing::debug;
use ::tracing::info;
use ::tracing::warn;

/// Path to Infoportal UI
const INFOPORTAL: &str = "infoportal";
const INFOPORTAL_V2: &str = "infoportal2";

/// The server struct contains the configuration of the server.
pub struct Server {}

/// The server implementation.
impl Server {
    /// Creates s new server instance with given configuration.
    /// # Returns
    /// * `Server` - The server.
    pub(crate) fn new() -> Server {
        Server {}
    }

    /// Starts the server.
    /// # Returns
    /// `io::Result<()>` - The result of the server start.
    /// # Panics
    /// If the server can't be started.
    pub(crate) async fn start(&self) -> IoResult<()> {
        let start = Instant::now();

        let aquarius = create_app_data().await.unwrap();
        let (rl_max_requests, rl_interval) = CONFIG.get_rate_limiter_config();
        let secret_key = Key::generate();
        let http_app_content_path = CONFIG.http_app_content_path.clone();

        let worker_count = Arc::new(Mutex::new(0));
        let prometheus = Self::get_prometheus();
        let prometheus_for_metrics = prometheus.clone();

        let user_pool_manager = Data::new(UserPoolManager::new(CONFIG.get_db_config()));

        let app_factory = move || {
            let mut count = worker_count.lock().unwrap();
            *count += 1;
            debug!(count = *count, "Created application HTTP worker:");

            // get app with some middlewares initialized
            Self::get_app(secret_key.clone(), rl_max_requests, rl_interval)
                // collect metrics about requests and responses
                .wrap(prometheus.clone())
                .app_data(aquarius.clone())
                .app_data(user_pool_manager.clone())
                .configure(rest_api::config)
                .configure(api_doc::config)
                .service(
                    Files::new(INFOPORTAL, http_app_content_path.clone())
                        .index_file("index.html")
                        .use_last_modified(true)
                        .use_etag(true)
                        .redirect_to_slash_directory(),
                )
                .service(
                    Files::new(INFOPORTAL_V2, http_app_content_path.clone())
                        .index_file("index_v2.html")
                        .use_last_modified(true)
                        .use_etag(true)
                        .redirect_to_slash_directory(),
                )
                // redirect from / to /infoportal
                .service(web::redirect("/", INFOPORTAL))
        };

        let mut app_http_server = HttpServer::new(app_factory)
            // always bind to http
            .bind(CONFIG.get_http_bind())?;

        // also bind to https if config is available
        if let Some(rustls_cfg) = Self::get_rustls_config() {
            let https_bind = CONFIG.get_https_bind();
            app_http_server = app_http_server.bind_rustls_0_23(https_bind, rustls_cfg)?;
        }

        // configure number of workers if env. variable is set
        if let Some(workers) = CONFIG.http_workers {
            app_http_server = app_http_server.workers(workers);
        }
        let app_server = app_http_server.run();

        // start a separate server for prometheus metrics
        let prometheus_data = Data::new(prometheus_for_metrics);
        let metrics_server = HttpServer::new(move || {
            debug!("Created metrics HTTP worker");
            App::new()
                .app_data(prometheus_data.clone())
                .route("/metrics", web::get().to(metrics_handler))
        })
        .bind(CONFIG.get_metrics_bind())?;
        let metrics_server = metrics_server.workers(1).run();

        info!(elapsed = ?start.elapsed(), "Infoportal started:");
        try_join!(app_server, metrics_server)?;
        Ok(())
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
        let expiration = Duration::from_secs(60 * 60 * 24 * 2); // 2 days
        let identity_mw = IdentityMiddleware::builder()
            .visit_deadline(Some(expiration))
            .logout_behavior(LogoutBehavior::DeleteIdentityKeys)
            .build();
        App::new()
            // Install the identity framework first.
            .wrap(identity_mw)
            // adds support for HTTPS sessions
            .wrap(Self::get_session_middleware(secret_key, expiration))
            // adds support for rate limiting of HTTP requests
            .wrap(Self::get_rate_limiter(rl_max_requests, rl_interval))
            .wrap_fn(|req, srv| {
                srv.call(req).map(|res| {
                    res.map(|mut response| {
                        let headers = response.headers_mut();
                        headers.insert(
                            HeaderName::from_static("x-frame-options"),
                            HeaderValue::from_static("DENY"),
                        );
                        headers.insert(
                            HeaderName::from_static("x-content-type-options"),
                            HeaderValue::from_static("nosniff"),
                        );
                        headers.insert(
                            HeaderName::from_static("referrer-policy"),
                            HeaderValue::from_static("strict-origin-when-cross-origin"),
                        );
                        headers.insert(
                            HeaderName::from_static("content-security-policy"),
                            HeaderValue::from_static(
                                "default-src 'self'; script-src 'self' sdk.openui5.org; style-src 'self' 'unsafe-inline' sdk.openui5.org; img-src 'self' data: https:; connect-src 'self' sdk.openui5.org; font-src 'self' sdk.openui5.org",
                            ),
                        );
                        response
                    })
                })
            })
    }

    /// Returns a new SessionMiddleware instance.
    /// # Arguments
    /// * `secret_key` - The secret key used to encrypt the session cookie.
    /// # Returns
    /// `SessionMiddleware<CookieSessionStore>` - The session middleware.
    /// # Panics
    /// If the session middleware can't be created.
    fn get_session_middleware(secret_key: Key, expiration: Duration) -> SessionMiddleware<CookieSessionStore> {
        SessionMiddleware::builder(CookieSessionStore::default(), secret_key)
            .cookie_secure(true)
            .cookie_http_only(true)
            // allow the cookie only from the current domain
            .cookie_same_site(SameSite::Strict)
            .session_lifecycle(
                PersistentSession::default()
                    .session_ttl_extension_policy(TtlExtensionPolicy::OnEveryRequest)
                    .session_ttl(expiration.try_into().expect("a valid duration")),
            )
            .cookie_path("/".to_string())
            .cookie_name("session_id".to_string())
            .build()
    }

    /// Returns a new PrometheusMetrics instance.
    /// # Returns
    /// `Arc<PrometheusMetrics>` - The prometheus metrics.
    /// # Panics
    /// If the prometheus metrics can't be created.
    fn get_prometheus() -> Arc<PrometheusMetrics> {
        Arc::new(
            PrometheusMetricsBuilder::new("api")
                .registry(Registry::new())
                .build()
                .unwrap(),
        )
    }

    /// Returns a new RateLimiter instance.
    /// # Arguments
    /// * `max_requests` - The maximum number of requests in the given interval.
    /// * `interval` - The interval in seconds.
    /// # Returns
    /// `RateLimiter<InMemoryBackend, SimpleOutput, impl Fn(&ServiceRequest) -> Ready<Result<SimpleInput, Error>>>` - The rate limiter.
    /// # Panics
    /// If the rate limiter can't be created.
    fn get_rate_limiter(
        max_requests: u64,
        interval: u64,
    ) -> RateLimiter<InMemoryBackend, SimpleOutput, impl Fn(&ServiceRequest) -> Ready<Result<SimpleInput, Error>>> {
        let input = SimpleInputFunctionBuilder::new(Duration::from_secs(interval), max_requests)
            .peer_ip_key()
            .build();

        RateLimiter::builder(InMemoryBackend::builder().build(), input)
            .add_headers()
            .build()
    }

    /// Returns HTTPS server configuration if available.
    /// # Returns
    /// `Option<ServerConfig>` - The HTTPS server configuration.
    /// # Panics
    /// If the HTTPS server configuration can't be created.
    /// # Remarks
    /// The HTTPS server configuration is only created if the certificate and private key files are available.
    /// The certificate and private key files are configured in the environment.
    fn get_rustls_config() -> Option<ServerConfig> {
        let cert_pem_path = Path::new(&CONFIG.https_cert_path);
        let key_pem_path = Path::new(&CONFIG.https_key_path);

        info!(
            path = %std::env::current_dir().unwrap().display(),
            "Working directory:"
        );

        if cert_pem_path.exists() && cert_pem_path.is_file() && key_pem_path.exists() && key_pem_path.is_file() {
            // load TLS key/cert files
            debug!(
                cert_path = &CONFIG.https_cert_path,
                key_path = &CONFIG.https_key_path,
                "Try to load TLS config:"
            );

            if let (Ok(cert_file), Ok(key_file)) = (File::open(cert_pem_path), File::open(key_pem_path)) {
                info!(
                    cert_path = &CONFIG.https_cert_path,
                    key_path = &CONFIG.https_key_path,
                    "TLS config loaded:"
                );
                let cert_reader = &mut BufReader::new(cert_file);
                let cert_chain = certs(cert_reader)
                    .map(|cert| cert.expect("Failed to parse certificate from cert.pem"))
                    .collect();

                let key_reader = &mut BufReader::new(key_file);
                // convert files to key/cert objects
                let mut keys: Vec<PrivatePkcs8KeyDer> = pkcs8_private_keys(key_reader)
                    .map(|key| key.expect("Failed to parse PKCS8 private key from key.pem"))
                    .collect();

                // no keys could be parsed for each variant
                if keys.is_empty() {
                    warn!("Could not locate PKCS 8 private keys.");
                    return None;
                }

                // init server config builder with safe defaults
                let config = ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(cert_chain, PrivateKeyDer::Pkcs8(keys.remove(0)))
                    .expect("Failed to build TLS ServerConfig with provided cert/key");
                Some(config)
            } else {
                warn!(
                    cert_path = &CONFIG.https_cert_path,
                    key_path = &CONFIG.https_key_path,
                    "Can't open one or both files:",
                );
                None
            }
        } else {
            warn!(
                cert_path = &CONFIG.https_cert_path,
                key_path = &CONFIG.https_key_path,
                "One or both are not existing or are directories:",
            );
            None
        }
    }
}

/// Serves Prometheus metrics for the internal-only metrics server.
async fn metrics_handler(prometheus: Data<Arc<PrometheusMetrics>>) -> HttpResponse {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&prometheus.registry.gather(), &mut buffer).ok();
    encoder.encode(&::prometheus::gather(), &mut buffer).ok();
    HttpResponse::Ok().content_type(encoder.format_type()).body(buffer)
}

pub async fn create_app_data() -> Result<Data<Aquarius>, DbError> {
    Ok(Data::new(
        Aquarius::new(CONFIG.active_regatta_id, CONFIG.cache_ttl).await?,
    ))
}
