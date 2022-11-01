use crate::db::{
    aquarius::Aquarius,
    model::{Heat, HeatRegistration, Race, Regatta, Registration, Score},
};
use actix_web::{
    get,
    web::{Data, Json, Path},
};

#[get("/regattas")]
async fn get_regattas(data: Data<Aquarius>) -> Json<Vec<Regatta>> {
    let regattas = data.get_regattas().await.unwrap();
    Json(regattas)
}

#[get("/regattas/{id}")]
async fn get_regatta(path: Path<i32>, data: Data<Aquarius>) -> Json<Regatta> {
    let regatta_id = path.into_inner();
    let regatta = data.get_regatta(regatta_id).await.unwrap();
    Json(regatta)
}

#[get("/regattas/{id}/races")]
async fn get_races(path: Path<i32>, data: Data<Aquarius>) -> Json<Vec<Race>> {
    let regatta_id = path.into_inner();
    let races = data.get_races(regatta_id).await.unwrap();
    Json(races)
}

#[get("/races/{id}")]
async fn get_race(path: Path<i32>, data: Data<Aquarius>) -> Json<Race> {
    let race_id = path.into_inner();
    let race = data.get_race(race_id).await.unwrap();
    Json(race)
}

#[get("/races/{id}/registrations")]
async fn get_registrations(path: Path<i32>, data: Data<Aquarius>) -> Json<Vec<Registration>> {
    let race_id = path.into_inner();
    let races = data.get_registrations(race_id).await.unwrap();
    Json(races)
}

#[get("/regattas/{id}/heats")]
async fn get_heats(path: Path<i32>, data: Data<Aquarius>) -> Json<Vec<Heat>> {
    let regatta_id = path.into_inner();
    let heats = data.get_heats(regatta_id).await.unwrap();
    Json(heats)
}

#[get("/regattas/{id}/scoring")]
async fn get_scoring(path: Path<i32>, data: Data<Aquarius>) -> Json<Vec<Score>> {
    let regatta_id = path.into_inner();
    let scoring = data.get_scoring(regatta_id).await.unwrap();
    Json(scoring)
}

#[get("/heats/{id}/registrations")]
async fn get_heat_registrations(
    path: Path<i32>,
    data: Data<Aquarius>,
) -> Json<Vec<HeatRegistration>> {
    let heat_id = path.into_inner();
    let heat_registrations = data.get_heat_registrations(heat_id).await.unwrap();
    Json(heat_registrations)
}
