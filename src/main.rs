mod aquarius_db;
mod connection_manager;
mod rest_api;

use actix_files::Files;
use actix_web::{web::Data, App, HttpServer};
use bb8::Pool;
use connection_manager::TiberiusConnectionManager;
use log::info;
use std::{env, io::Result};
use tiberius::{AuthMethod, EncryptionLevel};

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting infopoint");

    let pool: Pool<TiberiusConnectionManager> = create_pool().await;
    let data = Data::new(pool);
    let http_port = get_http_port();

    info!("Starting HTTP server on port {http_port}");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::clone(&data))
            .service(rest_api::hello)
            .service(rest_api::regattas)
            .service(rest_api::heats)
            .service(rest_api::heat_registrations)
            .service(Files::new("/", "./static").show_files_listing())
            .service(Files::new("/ui", "./static/ui").index_file("index.html"))
    })
    .bind(("127.0.0.1", http_port))?
    .workers(4)
    .run()
    .await
}

async fn create_pool() -> Pool<TiberiusConnectionManager> {
    let config = create_config();
    let manager = TiberiusConnectionManager::new(config).unwrap();
    let pool = Pool::builder().max_size(20).build(manager).await.unwrap();
    pool
}

fn get_http_port() -> u16 {
    env::var("HTTP_PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap()
}

fn get_db_port() -> u16 {
    env::var("DB_PORT")
        .unwrap_or("1433".to_string())
        .parse()
        .unwrap()
}

fn get_db_host() -> String {
    env::var("DB_HOST").unwrap_or("8e835d.online-server.cloud".to_string())
}

fn get_db_name() -> String {
    env::var("DB_NAME").unwrap_or("Regatta_2022_Test".to_string())
}

fn get_db_user() -> String {
    env::var("DB_USER").unwrap_or("sa".to_string())
}

fn get_db_password() -> String {
    env::var("DB_PASSWORD").unwrap()
}

fn create_config() -> tiberius::Config {
    let db_host = get_db_host();
    let db_port = get_db_port();
    let db_name = get_db_name();
    let db_user = get_db_user();

    info!(
        "Database configuration: host={}, port={}, name={}, user={}",
        db_host, db_port, db_name, db_user
    );

    let mut config = tiberius::Config::new();
    config.host(db_host);
    config.port(db_port);
    config.database(db_name);
    config.authentication(AuthMethod::sql_server(db_user, get_db_password()));
    config.encryption(EncryptionLevel::NotSupported);
    config
}
