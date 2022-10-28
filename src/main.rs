mod api;
mod db;

use crate::api::server::Server;
use dotenv::dotenv;
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    Server::start().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{
        rest_api,
        server::{create_app_data, SCOPE_API},
    };
    use actix_web::{
        test,
        test::TestRequest,
        web::{scope, Data},
        App,
    };

    #[actix_web::test]
    async fn test_get_regattas() {
        dotenv().ok();
        let app_data = create_app_data().await;

        let app = test::init_service(
            App::new().service(
                scope(SCOPE_API)
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
                scope(SCOPE_API)
                    .service(rest_api::get_heats)
                    .app_data(Data::clone(&app_data)),
            ),
        )
        .await;

        let request = TestRequest::get()
            .uri("/api/regattas/12/heats")
            .to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
    }
}
