pub(crate) mod athlete;
pub(crate) mod authentication;
pub(crate) mod notification;

use crate::config::CONFIG;
use crate::{db::aquarius::Aquarius, http::ws};
use ::actix_identity::Identity;
use ::actix_web::{
    Error, Responder, Scope as ActixScope,
    error::{ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized},
    get,
    web::{Data, Json, Path, ServiceConfig},
};
use ::db::tiberius::create_client;
use ::db::{
    aquarius::model::{Club, Entry, Filters, Heat, Race, Regatta},
    timekeeper::{TimeStamp, TimeStrip},
};
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

// Clubs Endpoints

#[utoipa::path(
    description = "Get all participating clubs of a regatta.",
    context_path = PATH,
    responses(
        (status = 200, description = "Participating clubs", body = Vec<Club>),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/{regatta_id}/clubs")]
async fn get_participating_clubs(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let regatta_id = path.into_inner();
    let clubs = aquarius
        .get_participating_clubs(regatta_id, opt_user)
        .await
        .map_err(|err| {
            error!("{err}");
            ErrorInternalServerError(err)
        })?;
    Ok(Json(clubs))
}

#[utoipa::path(
    description = "Get all entries of a specific club in a regatta.",
    context_path = PATH,
    responses(
        (status = 200, description = "Club entries", body = Vec<Entry>),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/{regatta_id}/clubs/{club_id}/entries")]
async fn get_club_entries(
    ids: Path<(i32, i32)>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let ids = ids.into_inner();
    let entries = aquarius.get_club_entries(ids.0, ids.1, opt_user).await.map_err(|err| {
        error!("{err}");
        ErrorInternalServerError(err)
    })?;
    Ok(Json(entries))
}

#[utoipa::path(
    description = "Get a specific club participating in a regatta.",
    context_path = PATH,
    responses(
        (status = 200, description = "Club details", body = Club),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/{regatta_id}/clubs/{club_id}")]
async fn get_regatta_club(
    ids: Path<(i32, i32)>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let ids = ids.into_inner();
    let club = aquarius.get_regatta_club(ids.0, ids.1, opt_user).await.map_err(|err| {
        error!("{err}");
        ErrorInternalServerError(err)
    })?;
    Ok(Json(club))
}

// Misc Endpoints

#[utoipa::path(
    description = "Get the timestrip data for the active regatta. Requires authentication.",
    context_path = PATH,
    responses(
        (status = 200, description = "Timestrip data", body = Vec<TimeStamp>),
        (status = 401, description = "Unauthorized", body = String, example = "Unauthorized"),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/active/timestrip")]
async fn get_timestrip(opt_user: Option<Identity>) -> Result<impl Responder, Error> {
    if let Some(user) = opt_user
        && let Ok(id) = user.id()
        && id == "sa"
    {
        let client = create_client(&CONFIG.get_db_config()).await.map_err(|err| {
            error!("{err}");
            ErrorInternalServerError(err)
        })?;
        let timestrip = TimeStrip::load(client).await.map_err(|err| {
            error!("{err}");
            ErrorInternalServerError(err)
        })?;
        return Ok(Json(timestrip.time_stamps));
    }
    Err(ErrorUnauthorized("Unauthorized"))
}

#[utoipa::path(
    description = "Get statistics for a regatta. Requires authentication.",
    context_path = PATH,
    responses(
        (status = 200, description = "Regatta statistics"),
        (status = 401, description = "Unauthorized", body = String, example = "Unauthorized"),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/{regatta_id}/statistics")]
async fn get_statistics(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    if opt_user.is_some() {
        let regatta_id = path.into_inner();
        let stats = aquarius.query_statistics(regatta_id).await.map_err(|err| {
            error!("{err}");
            ErrorInternalServerError(err)
        })?;
        Ok(Json(stats))
    } else {
        Err(ErrorUnauthorized("Unauthorized"))
    }
}

#[utoipa::path(
    description = "Calculate scoring for a regatta. Requires authentication.",
    context_path = PATH,
    responses(
        (status = 200, description = "Calculated scoring data"),
        (status = 401, description = "Unauthorized", body = String, example = "Unauthorized"),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/{regatta_id}/calculateScoring")]
async fn calculate_scoring(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    if opt_user.is_some() {
        let regatta_id = path.into_inner();
        let scoring = aquarius.calculate_scoring(regatta_id).await.map_err(|err| {
            error!("{err}");
            ErrorInternalServerError(err)
        })?;
        Ok(Json(scoring))
    } else {
        Err(ErrorUnauthorized("Unauthorized"))
    }
}

#[utoipa::path(
    description = "Get the schedule for a regatta.",
    context_path = PATH,
    responses(
        (status = 200, description = "Regatta schedule"),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/{regatta_id}/schedule")]
async fn get_schedule(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let regatta_id = path.into_inner();
    let schedule = aquarius.query_schedule(regatta_id, opt_user).await.map_err(|err| {
        error!("{err}");
        ErrorInternalServerError(err)
    })?;
    Ok(Json(schedule))
}

/// Configure the REST API. This will add all REST API endpoints to the service configuration.
pub(crate) fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        ActixScope::new(PATH)
            .service(athlete::get_athlete)
            .service(get_regatta_club)
            .service(get_club_entries)
            .service(athlete::get_athlete_entries)
            .service(get_participating_clubs)
            .service(athlete::get_participating_athletes)
            .service(get_active_regatta)
            .service(get_race)
            .service(get_races)
            .service(get_heats)
            .service(get_filters)
            .service(get_heat)
            .service(calculate_scoring)
            .service(get_statistics)
            .service(get_schedule)
            .service(get_timestrip)
            .service(notification::get_notifications)
            .service(notification::notification_read)
            .service(authentication::login)
            .service(authentication::identity)
            .service(authentication::logout)
            .service(ws::index),
    );
}
