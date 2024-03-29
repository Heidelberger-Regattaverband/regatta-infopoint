mod config;
mod db;
mod http;

use crate::http::server::Server;
use config::Config;
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    let server = Server::new(Config::get());
    server.start().await
}

#[cfg(test)]
mod tests {
    use crate::http::{
        rest_api::{self, PATH},
        server::create_app_data,
    };
    use actix_identity::IdentityMiddleware;
    use actix_web::{
        test,
        test::TestRequest,
        web::{scope, Data},
        App,
    };
    use dotenv::dotenv;

    #[actix_web::test]
    async fn test_get_regattas() {
        dotenv().ok();

        let app_data = create_app_data().await;

        let app = test::init_service(
            App::new().service(
                scope(PATH)
                    .service(rest_api::get_regattas)
                    .app_data(Data::clone(&app_data)),
            ),
        )
        .await;

        let request = TestRequest::get().uri("/api/regattas").to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
    }

    #[actix_web::test]
    async fn test_get_heats() {
        dotenv().ok();

        let app_data = create_app_data().await;

        let app = test::init_service(
            App::new().service(
                scope(PATH)
                    .service(rest_api::get_heats)
                    .wrap(IdentityMiddleware::default())
                    .app_data(Data::clone(&app_data)),
            ),
        )
        .await;

        let request = TestRequest::get().uri("/api/regattas/12/heats").to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
    }
}
