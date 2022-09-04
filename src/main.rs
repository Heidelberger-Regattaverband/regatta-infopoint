mod aquarius_db;
mod connection_manager;
mod rest_api;

use actix_web::{web::Data, App, HttpServer};
use bb8::Pool;
use connection_manager::TiberiusConnectionManager;
use std::io::Result;
use tiberius::{AuthMethod, Config, EncryptionLevel};

#[actix_web::main]
async fn main() -> Result<()> {
    let manager = TiberiusConnectionManager::new(create_config()).unwrap();
    let db_pool = Pool::builder().max_size(20).build(manager).await.unwrap();
    let data = Data::new(db_pool);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::clone(&data))
            .service(rest_api::hello)
            .service(rest_api::regattas)
            .service(rest_api::heats)
            .service(rest_api::heat_registrations)
    })
    .bind(("127.0.0.1", 8080))?
    .workers(4)
    .run()
    .await
}

fn create_config() -> Config {
    let mut config = Config::new();
    config.host("8e835d.online-server.cloud");
    config.port(1433);
    config.authentication(AuthMethod::sql_server("SA", "Regatta4HD"));
    config.database("Regatta_2022_Test");
    config.encryption(EncryptionLevel::NotSupported);
    config
}
