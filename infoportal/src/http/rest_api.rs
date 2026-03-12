pub(crate) mod athlete;
pub(crate) mod authentication;
pub(crate) mod club;
pub(crate) mod misc;
pub(crate) mod monitoring;
pub(crate) mod notification;
pub(crate) mod race;

use ::actix_identity::Identity;
use ::actix_web::{
    Error, Responder, Scope as ActixScope,
    error::{ErrorInternalServerError, ErrorNotFound},
    get,
    web::{Data, Json, Path, ServiceConfig},
};
use ::db::aquarius::Aquarius;
use ::db::aquarius::model::{Filters, Heat, Regatta};
use ::db::tiberius::TiberiusPool;
use ::db::tiberius::user_pool::UserPoolManager;
use ::std::sync::Arc;
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
    regatta_id: Path<i32>,
    aquarius: Data<Aquarius>,
    identity: Option<Identity>,
) -> Result<impl Responder, Error> {
    let filters = aquarius
        .get_filters(regatta_id.into_inner(), identity.is_some())
        .await
        .map_err(|err| {
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
async fn get_active_regatta(aquarius: Data<Aquarius>, identity: Option<Identity>) -> Result<impl Responder, Error> {
    let regatta = aquarius.get_active_regatta(identity.is_some()).await.map_err(|err| {
        error!("{err}");
        ErrorInternalServerError(err)
    })?;
    if regatta.is_none() {
        return Err(ErrorNotFound("No active regatta found"));
    }
    Ok(Json(regatta))
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
    regatta_id: Path<i32>,
    aquarius: Data<Aquarius>,
    identity: Option<Identity>,
) -> Result<impl Responder, Error> {
    let heats = aquarius
        .get_heats(regatta_id.into_inner(), identity.is_some())
        .await
        .map_err(|err| {
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
    heat_id: Path<i32>,
    aquarius: Data<Aquarius>,
    identity: Option<Identity>,
) -> Result<impl Responder, Error> {
    let heat = aquarius
        .get_heat(heat_id.into_inner(), identity.is_some())
        .await
        .map_err(|err| {
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
            .service(race::get_race)
            .service(race::get_races)
            .service(get_heats)
            .service(get_filters)
            .service(get_heat)
            .service(misc::calculate_scoring)
            .service(misc::get_statistics)
            .service(misc::get_schedule)
            .service(misc::get_timestrip)
            .service(notification::get_visible_notifications)
            .service(notification::get_all_notifications)
            .service(notification::create_notification)
            .service(notification::update_notification)
            .service(notification::delete_notification)
            .service(notification::notification_read)
            .service(authentication::login)
            .service(authentication::identity)
            .service(authentication::logout)
            .service(monitoring::index),
    );
}

/// Helper function to get the user-specific connection pool for a given identity. Returns an error if no pool is found.
///
/// Arguments:
/// * `identity` - The identity of the user.
/// * `user_pool_manager` - The user pool manager.
///
/// Returns:
/// * `Result<Arc<TiberiusPool>, Error>` - The user-specific connection pool or an error.
async fn get_user_pool(
    identity: &Identity,
    user_pool_manager: &Data<UserPoolManager>,
) -> Result<Arc<TiberiusPool>, Error> {
    user_pool_manager
        .get_pool(&identity.id()?)
        .await
        .ok_or_else(|| ErrorInternalServerError("No connection pool found"))
}
