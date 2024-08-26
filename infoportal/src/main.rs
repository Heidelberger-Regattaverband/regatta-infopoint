mod config;
mod db;
mod http;

use aquarius::db::tiberius::TiberiusPool;
use config::Config;
use http::server::Server;
use std::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    TiberiusPool::init(
        Config::get().get_db_config(),
        Config::get().db_pool_max_size,
        Config::get().db_pool_min_idle,
    )
    .await;
    Server::new().start().await
}

#[cfg(test)]
mod tests {
    use crate::{
        config::Config,
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
    use aquarius::db::tiberius::TiberiusPool;
    use dotenv::dotenv;

    #[tokio_shared_rt::test(shared)]
    async fn test_get_regattas() {
        dotenv().ok();
        TiberiusPool::init(
            Config::get().get_db_config(),
            Config::get().db_pool_max_size,
            Config::get().db_pool_min_idle,
        )
        .await;

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

    #[tokio_shared_rt::test(shared)]
    async fn test_get_heats() {
        dotenv().ok();
        TiberiusPool::init(
            Config::get().get_db_config(),
            Config::get().db_pool_max_size,
            Config::get().db_pool_min_idle,
        )
        .await;

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