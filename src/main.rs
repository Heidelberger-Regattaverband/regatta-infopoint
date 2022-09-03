mod aquarius_db;
mod connection_manager;
mod rest_api;

use actix_web::{web::Data, App, HttpServer};
use bb8::Pool;
use connection_manager::{TiberiusConnectionManager, TiberiusPool};

// #[async_std::main]
// async fn main() -> Result<()> {
//     let mut client = aquarius_db::create_client(aquarius_db::create_config()).await?;
//     let heats = aquarius_db::get_heats(&mut client).await?;
//     println!("Heats count: {}", heats.len());

//     let heat_id = heats.get(1).unwrap().id;
//     let heat_registrations = aquarius_db::get_heat_registrations(&mut client, heat_id).await?;
//     println!("heat_registrations count: {}", heat_registrations.len());

//     Ok(())
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let manager: TiberiusConnectionManager =
        TiberiusConnectionManager::new(aquarius_db::create_config()).unwrap();
    let db_pool: TiberiusPool = Pool::builder().max_size(5).build(manager).await.unwrap();
    let data: Data<TiberiusPool> = Data::new(db_pool);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::clone(&data))
            .service(rest_api::hello)
            .service(rest_api::heats)
            .service(rest_api::heat_registrations)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
