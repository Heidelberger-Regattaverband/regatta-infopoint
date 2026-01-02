mod config;
mod db;
mod http;
mod peak_alloc;

use ::db::tiberius::TiberiusPool;
use http::server::Server;
use peak_alloc::PeakAlloc;
use std::io::Result;

use crate::config::CONFIG;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[tokio::main]
async fn main() -> Result<()> {
    TiberiusPool::init(CONFIG.get_db_config(), CONFIG.db_pool_max_size, CONFIG.db_pool_min_idle).await;
    Server::new().start().await
}

#[cfg(test)]
mod tests {
    use crate::config::CONFIG;
    use crate::http::{
        rest_api::{self, PATH},
        server::create_app_data,
    };
    use actix_identity::IdentityMiddleware;
    use actix_web::{
        App, test,
        test::TestRequest,
        web::{Data, scope},
    };
    use db::tiberius::TiberiusPool;
    use dotenv::dotenv;

    #[tokio_shared_rt::test(shared)]
    async fn test_get_regattas() {
        dotenv().ok();
        TiberiusPool::init(CONFIG.get_db_config(), CONFIG.db_pool_max_size, CONFIG.db_pool_min_idle).await;

        let app_data = create_app_data().await.unwrap();

        let app = test::init_service(
            App::new().service(
                scope(PATH)
                    .service(rest_api::get_active_regatta)
                    .wrap(IdentityMiddleware::default())
                    .app_data(Data::clone(&app_data)),
            ),
        )
        .await;

        let request = TestRequest::get().uri("/api/active_regatta").to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_get_heats() {
        dotenv().ok();
        TiberiusPool::init(CONFIG.get_db_config(), CONFIG.db_pool_max_size, CONFIG.db_pool_min_idle).await;

        let app_data = create_app_data().await.unwrap();

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
