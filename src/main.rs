mod aquarius_db;
mod db;
mod rest_api;

use actix_files::Files;
use actix_web::{web::Data, App, HttpServer};
use log::info;
use std::{env, io::Result};

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting infopoint");

    let pool = db::create_pool().await;
    let data = Data::new(pool);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::clone(&data))
            .service(rest_api::regattas)
            .service(rest_api::heats)
            .service(rest_api::heat_registrations)
            .service(Files::new("/", "./static").show_files_listing())
            .service(Files::new("/ui", "./static/ui").index_file("index.html"))
    })
    .bind(get_http_bind())?
    .workers(4)
    .run()
    .await
}

fn get_http_bind() -> (String, u16) {
    let port = env::var("HTTP_PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap();
    let host = env::var("HTTP_BIND").unwrap_or("127.0.0.1".to_string());
    info!("HTTP server is listening on: {host}:{port}");

    (host, port)
}
