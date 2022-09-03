mod aquarius_db;
mod rest_api;

use actix_web::{App, HttpServer};

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
    HttpServer::new(|| {
        App::new()
            .service(rest_api::hello)
            .service(rest_api::heats)
            .service(rest_api::heat_registrations)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
