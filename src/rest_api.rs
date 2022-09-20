use crate::{
    db::aquarius::{self, Heat, HeatRegistration, Regatta},
    db::TiberiusPool,
};

use actix_web::{
    get,
    web::{Data, Json, Path},
};

#[get("/api/regattas")]
async fn get_regattas(data: Data<TiberiusPool>) -> Json<Vec<Regatta>> {
    let mut client = data.get().await.unwrap();
    let regattas = aquarius::get_regattas(&mut client).await.unwrap();
    Json(regattas)
}

#[get("/api/regattas/{id}")]
async fn get_regatta(path: Path<i32>, data: Data<TiberiusPool>) -> Json<Regatta> {
    let regatta_id = path.into_inner();
    let mut client = data.get().await.unwrap();
    let regatta = aquarius::get_regatta(&mut client, regatta_id)
        .await
        .unwrap();
    Json(regatta)
}

#[get("/api/regattas/{id}/heats")]
async fn get_heats(path: Path<i32>, data: Data<TiberiusPool>) -> Json<Vec<Heat>> {
    let regatta_id = path.into_inner();
    let mut client = data.get().await.unwrap();
    let heats = aquarius::get_heats(&mut client, regatta_id).await.unwrap();
    Json(heats)
}

#[get("/api/heats/{id}/registrations")]
async fn get_heat_registrations(
    path: Path<i32>,
    data: Data<TiberiusPool>,
) -> Json<Vec<HeatRegistration>> {
    let heat_id = path.into_inner();
    let mut client = data.get().await.unwrap();
    let heat_registrations = aquarius::get_heat_registrations(&mut client, heat_id)
        .await
        .unwrap();
    Json(heat_registrations)
}
