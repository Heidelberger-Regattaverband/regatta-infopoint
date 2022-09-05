mod aquarius_db;
mod connection_manager;
mod rest_api;

use actix_files::Files;
use actix_web::{web::Data, App, HttpServer};
use bb8::Pool;
use connection_manager::TiberiusConnectionManager;
use std::{env::var, io::Result};
use tiberius::{AuthMethod, Config, EncryptionLevel};

#[actix_web::main]
async fn main() -> Result<()> {
    let config = create_config();
    let manager = TiberiusConnectionManager::new(config).unwrap();
    let db_pool = Pool::builder().max_size(20).build(manager).await.unwrap();
    let data = Data::new(db_pool);
    let http_port = get_http_port();

    println!("Starting HTTP server on port {http_port}");

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

fn get_http_port() -> u16 {
    var("HTTP_PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap()
}

fn get_db_port() -> u16 {
    var("DB_PORT")
        .unwrap_or("1433".to_string())
        .parse()
        .unwrap()
}

fn get_db_host() -> String {
    var("DB_HOST").unwrap_or("8e835d.online-server.cloud".to_string())
}

fn get_db_name() -> String {
    var("DB_NAME").unwrap_or("Regatta_2022_Test".to_string())
}

fn create_config() -> Config {
    let db_host = get_db_host();
    let db_port = get_db_port();
    let db_name = get_db_name();

    println!(
        "Database configuration: host={}, port={}, name={}",
        db_host, db_port, db_name
    );

    let mut config = Config::new();
    config.host(db_host);
    config.port(db_port);
    config.database(db_name);
    config.authentication(AuthMethod::sql_server("SA", "Regatta4HD"));
    config.encryption(EncryptionLevel::NotSupported);
    config
}
