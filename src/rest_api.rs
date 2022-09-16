use crate::{
    aquarius_db::{self, Heat, HeatRegistration, Regatta},
    db::TiberiusPool,
};

use actix_web::{
    get,
    web::{Data, Json, Path},
};

#[get("/api/regattas")]
async fn get_regattas(data: Data<TiberiusPool>) -> Json<Vec<Regatta>> {
    let mut client = data.get().await.unwrap();
    let regattas = aquarius_db::get_regattas(&mut client).await.unwrap();
    Json(regattas)
}

#[get("/api/heats")]
async fn get_heats(data: Data<TiberiusPool>) -> Json<Vec<Heat>> {
    let mut client = data.get().await.unwrap();
    let heats = aquarius_db::get_heats(&mut client).await.unwrap();
    Json(heats)
}

#[get("/api/heats/{id}/registrations")]
async fn get_heat_registrations(
    path: Path<i32>,
    data: Data<TiberiusPool>,
) -> Json<Vec<HeatRegistration>> {
    let heat_id = path.into_inner();
    let mut client = data.get().await.unwrap();
    let heat_registrations = aquarius_db::get_heat_registrations(&mut client, heat_id)
        .await
        .unwrap();
    Json(heat_registrations)
}
