use crate::aquarius_db::{
    create_client, create_config, get_heat_registrations, get_heats, Heat, HeatRegistration,
};
use actix_web::{
    get,
    web::{Json, Path},
    HttpResponse, Responder,
};

#[get("/api")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/api/heats")]
async fn heats() -> Json<Vec<Heat>> {
    let mut client = create_client(create_config()).await.unwrap();
    let heats = get_heats(&mut client).await.unwrap();

    Json(heats)
}

#[get("/api/heat_registrations/{id}")]
async fn heat_registrations(path: Path<i32>) -> Json<Vec<HeatRegistration>> {
    let heat_id = path.into_inner();

    let mut client = create_client(create_config()).await.unwrap();
    let heat_registrations = get_heat_registrations(&mut client, heat_id).await.unwrap();

    Json(heat_registrations)
}
