use crate::{
    aquarius_db::{get_heat_registrations, get_heats, Heat, HeatRegistration},
    connection_manager::TiberiusPool,
};

use actix_web::{
    get,
    web::Data,
    web::{Json, Path},
    HttpResponse, Responder,
};

#[get("/api")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/api/heats")]
async fn heats(data: Data<TiberiusPool>) -> Json<Vec<Heat>> {
    let mut client = data.get().await.unwrap();

    let heats = get_heats(&mut client).await.unwrap();

    Json(heats)
}

#[get("/api/heat_registrations/{id}")]
async fn heat_registrations(path: Path<i32>, data: Data<TiberiusPool>) -> Json<Vec<HeatRegistration>> {
    let heat_id = path.into_inner();
    let mut client = data.get().await.unwrap();

    let heat_registrations = get_heat_registrations(&mut client, heat_id).await.unwrap();

    Json(heat_registrations)
}
