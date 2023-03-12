use crate::{
    db::{
        aquarius::Aquarius,
        model::{Heat, HeatRegistration, Kiosk, Race, Regatta, Registration, Score, Statistics},
    },
    http::{
        auth::{Credentials, User},
        monitor::Monitor,
    },
};
use actix_identity::Identity;
use actix_web::{
    error::InternalError,
    get, post,
    web::{Data, Json, Path, Query},
    Error, HttpMessage, HttpRequest, HttpResponse, Responder,
};
use serde::Deserialize;

#[get("/monitor")]
async fn monitor(aquarius: Data<Aquarius>) -> Json<Monitor> {
    let pool = aquarius.pool.state();
    Json(Monitor::new(pool))
}

#[get("/regattas")]
async fn get_regattas(aquarius: Data<Aquarius>) -> Json<Vec<Regatta>> {
    Json(aquarius.get_regattas().await)
}

#[get("/active_regatta")]
async fn get_active_regatta(aquarius: Data<Aquarius>) -> Json<Regatta> {
    Json(aquarius.get_active_regatta().await)
}

#[get("/regattas/{id}")]
async fn get_regatta(path: Path<i32>, aquarius: Data<Aquarius>) -> Json<Regatta> {
    let regatta_id = path.into_inner();
    Json(aquarius.get_regatta(regatta_id).await)
}

#[get("/regattas/{id}/races")]
async fn get_races(path: Path<i32>, aquarius: Data<Aquarius>) -> Json<Vec<Race>> {
    let regatta_id = path.into_inner();
    Json(aquarius.get_races(regatta_id).await)
}

#[get("/regattas/{id}/statistics")]
async fn get_statistics(path: Path<i32>, aquarius: Data<Aquarius>) -> Json<Statistics> {
    let regatta_id = path.into_inner();
    Json(aquarius.get_statistics(regatta_id).await)
}

#[get("/races/{id}")]
async fn get_race(path: Path<i32>, aquarius: Data<Aquarius>) -> Json<Race> {
    let race_id = path.into_inner();
    Json(aquarius.get_race(race_id).await)
}

#[get("/races/{id}/registrations")]
async fn get_registrations(path: Path<i32>, aquarius: Data<Aquarius>) -> Json<Vec<Registration>> {
    let race_id = path.into_inner();
    Json(aquarius.get_registrations(race_id).await)
}

#[get("/regattas/{id}/heats")]
async fn get_heats(path: Path<i32>, odata_params: Query<OData>, aquarius: Data<Aquarius>) -> Json<Vec<Heat>> {
    let regatta_id = path.into_inner();
    let odata = odata_params.into_inner();
    Json(aquarius.get_heats(regatta_id, odata.filter).await)
}

#[get("/regattas/{id}/kiosk")]
async fn get_kiosk(path: Path<i32>, aquarius: Data<Aquarius>) -> Json<Kiosk> {
    let regatta_id = path.into_inner();
    Json(aquarius.get_kiosk(regatta_id).await)
}

#[get("/regattas/{id}/scoring")]
async fn get_scoring(path: Path<i32>, aquarius: Data<Aquarius>) -> Json<Vec<Score>> {
    let regatta_id = path.into_inner();
    Json(aquarius.get_scoring(regatta_id).await)
}

#[get("/heats/{id}/registrations")]
async fn get_heat_registrations(path: Path<i32>, aquarius: Data<Aquarius>) -> Json<Vec<HeatRegistration>> {
    let heat_id = path.into_inner();
    Json(aquarius.get_heat_registrations(heat_id).await)
}

#[post("/login")]
async fn login(credentials: Json<Credentials>, request: HttpRequest) -> Result<impl Responder, Error> {
    match User::authenticate(credentials.into_inner()) {
        Ok(user) => {
            let response = format!("Welcome {}", user.name);
            Identity::login(&request.extensions(), user.name).unwrap();
            Ok(response)
        }
        Err(err) => Err(InternalError::from_response("", err).into()),
    }
}

#[post("/logout")]
async fn logout(user: Identity) -> impl Responder {
    user.logout();
    HttpResponse::Ok()
}

#[get("/identity")]
async fn identity(user: Option<Identity>) -> Result<impl Responder, Error> {
    if let Some(user) = user {
        Ok(user.id().unwrap())
    } else {
        Err(InternalError::from_response("", HttpResponse::Unauthorized().json("Unauthorized")).into())
    }
}

#[derive(Debug, Deserialize)]
struct OData {
    // #[serde(rename = "$expand")]
    // expand: Option<String>,
    #[serde(rename = "$filter")]
    filter: Option<String>,
}
