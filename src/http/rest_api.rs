use crate::db::{
    aquarius::Aquarius,
    model::{Heat, HeatRegistration, Kiosk, Race, Regatta, Registration, Score, Statistics},
};
use crate::http::monitor::Monitor;
use actix_web::{
    get,
    web::{Data, Json, Path, Query},
};
use serde::Deserialize;

#[get("/monitor")]
async fn monitor(data: Data<Aquarius>) -> Json<Monitor> {
    let pool = data.pool.state();
    Json(Monitor::new(pool))
}

#[get("/regattas")]
async fn get_regattas(data: Data<Aquarius>) -> Json<Vec<Regatta>> {
    Json(data.get_regattas().await)
}

#[get("/regattas/{id}")]
async fn get_regatta(path: Path<i32>, data: Data<Aquarius>) -> Json<Regatta> {
    let regatta_id = path.into_inner();
    Json(data.get_regatta(regatta_id).await)
}

#[get("/regattas/{id}/races")]
async fn get_races(path: Path<i32>, data: Data<Aquarius>) -> Json<Vec<Race>> {
    let regatta_id = path.into_inner();
    Json(data.get_races(regatta_id).await)
}

#[get("/regattas/{id}/statistics")]
async fn get_statistics(path: Path<i32>, data: Data<Aquarius>) -> Json<Statistics> {
    let regatta_id = path.into_inner();
    Json(data.get_statistics(regatta_id).await)
}

#[get("/races/{id}")]
async fn get_race(path: Path<i32>, data: Data<Aquarius>) -> Json<Race> {
    let race_id = path.into_inner();
    Json(data.get_race(race_id).await)
}

#[get("/races/{id}/registrations")]
async fn get_registrations(path: Path<i32>, data: Data<Aquarius>) -> Json<Vec<Registration>> {
    let race_id = path.into_inner();
    Json(data.get_registrations(race_id).await)
}

#[get("/regattas/{id}/heats")]
async fn get_heats(path: Path<i32>, odata_params: Query<OData>, data: Data<Aquarius>) -> Json<Vec<Heat>> {
    let regatta_id = path.into_inner();
    let odata = odata_params.into_inner();
    Json(data.get_heats(regatta_id, odata.filter).await)
}

#[get("/regattas/{id}/kiosk")]
async fn get_kiosk(path: Path<i32>, data: Data<Aquarius>) -> Json<Kiosk> {
    let regatta_id = path.into_inner();
    Json(data.get_kiosk(regatta_id).await)
}

#[get("/regattas/{id}/scoring")]
async fn get_scoring(path: Path<i32>, data: Data<Aquarius>) -> Json<Vec<Score>> {
    let regatta_id = path.into_inner();
    Json(data.get_scoring(regatta_id).await)
}

#[get("/heats/{id}/registrations")]
async fn get_heat_registrations(path: Path<i32>, data: Data<Aquarius>) -> Json<Vec<HeatRegistration>> {
    let heat_id = path.into_inner();
    Json(data.get_heat_registrations(heat_id).await)
}

#[derive(Debug, Deserialize)]
struct OData {
    // #[serde(rename = "$expand")]
    // expand: Option<String>,
    #[serde(rename = "$filter")]
    filter: Option<String>,
}
