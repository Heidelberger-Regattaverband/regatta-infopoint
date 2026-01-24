pub(crate) mod athlete;
pub(crate) mod authentication;
pub(crate) mod club;
pub(crate) mod misc;
pub(crate) mod monitoring;
pub(crate) mod notification;

use crate::db::aquarius::Aquarius;
use ::actix_identity::Identity;
use ::actix_web::{
    Error, Responder, Scope as ActixScope,
    error::{ErrorInternalServerError, ErrorNotFound},
    get,
    web::{Data, Json, Path, ServiceConfig},
};
use ::db::aquarius::model::{Filters, Heat, Race, Regatta};
use ::tracing::error;

/// Path to REST API
pub(crate) const PATH: &str = "/api";
const INTERNAL_SERVER_ERROR: &str = "Internal server error";

// Filters Endpoints
#[utoipa::path(
    context_path = PATH,
    responses(
        (status = 200, description = "Filters for <regatta_id>", body = Filters),
        (status = 404, description = "Regatta not found"),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/{regatta_id}/filters")]
async fn get_filters(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let regatta_id = path.into_inner();
    let filters = aquarius.get_filters(regatta_id, opt_user).await.map_err(|err| {
        error!("{err}");
        ErrorInternalServerError(err)
    })?;
    Ok(Json(filters))
}

// Regatta Endpoints
#[utoipa::path(
    description = "Get the currently active regatta. If no regatta is active, a 404 error is returned.",
    context_path = PATH,
    responses(
        (status = 200, description = "Active regatta found", body = Regatta),
        (status = 404, description = "No active regatta found", body = String),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/active_regatta")]
async fn get_active_regatta(aquarius: Data<Aquarius>, opt_user: Option<Identity>) -> Result<impl Responder, Error> {
    let regatta = aquarius.get_active_regatta(opt_user).await.map_err(|err| {
        error!("{err}");
        ErrorInternalServerError(err)
    })?;
    if regatta.is_none() {
        return Err(ErrorNotFound("No active regatta found"));
    }
    Ok(Json(regatta))
}

// Races Endpoints
#[utoipa::path(
    description = "Get all races of a regatta.",
    context_path = PATH,
    responses(
        (status = 200, description = "Races of <regatta_id>", body = Vec<Race>),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/{regatta_id}/races")]
async fn get_races(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let regatta_id = path.into_inner();
    let races = aquarius.get_races(regatta_id, opt_user).await.map_err(|err| {
        error!("{err}");
        ErrorInternalServerError(err)
    })?;
    Ok(Json(races))
}

#[utoipa::path(
    description = "Get a race with its heats and entries.",
    context_path = PATH,
    responses(
        (status = 200, description = "Race found", body = Race),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/races/{race_id}")]
async fn get_race(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let race_id = path.into_inner();
    let race = aquarius
        .get_race_heats_entries(race_id, opt_user)
        .await
        .map_err(|err| {
            error!("{err}");
            ErrorInternalServerError(err)
        })?;
    Ok(Json(race))
}

// Heats Endpoints

#[utoipa::path(
    description = "Get all heats of a regatta.",
    context_path = PATH,
    responses(
        (status = 200, description = "Heats of regatta", body = Vec<Heat>),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/{regatta_id}/heats")]
async fn get_heats(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let regatta_id = path.into_inner();
    let heats = aquarius.get_heats(regatta_id, opt_user).await.map_err(|err| {
        error!("{err}");
        ErrorInternalServerError(err)
    })?;
    Ok(Json(heats))
}

#[utoipa::path(
    description = "Get a specific heat by ID.",
    context_path = PATH,
    responses(
        (status = 200, description = "Heat found", body = Heat),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/heats/{id}")]
async fn get_heat(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let heat_id = path.into_inner();
    let heat = aquarius.get_heat(heat_id, opt_user).await.map_err(|err| {
        error!("{err}");
        ErrorInternalServerError(err)
    })?;
    Ok(Json(heat))
}

/// Configure the REST API. This will add all REST API endpoints to the service configuration.
pub(crate) fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        ActixScope::new(PATH)
            .service(club::get_regatta_club)
            .service(club::get_club_entries)
            .service(club::get_participating_clubs)
            .service(athlete::get_athlete)
            .service(athlete::get_athlete_entries)
            .service(athlete::get_participating_athletes)
            .service(get_active_regatta)
            .service(get_race)
            .service(get_races)
            .service(get_heats)
            .service(get_filters)
            .service(get_heat)
            .service(misc::calculate_scoring)
            .service(misc::get_statistics)
            .service(misc::get_schedule)
            .service(misc::get_timestrip)
            .service(notification::get_notifications)
            .service(notification::create_notification)
            .service(notification::update_notification)
            .service(notification::notification_read)
            .service(authentication::login)
            .service(authentication::identity)
            .service(authentication::logout)
            .service(monitoring::index),
    );
}
