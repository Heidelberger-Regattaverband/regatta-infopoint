mod config;
mod db;
mod http;

use db::tiberius::TiberiusPool;
use http::server::Server;
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    TiberiusPool::init().await;
    Server::new().start().await
}

#[cfg(test)]
mod tests {
    use crate::{
        db::tiberius::TiberiusPool,
        http::{
            rest_api::{self, PATH},
            server::create_app_data,
        },
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
        TiberiusPool::init().await;

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
        TiberiusPool::init().await;

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
