use crate::config::CONFIG;
use crate::db::aquarius::Aquarius;
use crate::http::rest_api::INTERNAL_SERVER_ERROR;
use crate::http::rest_api::PATH;
use ::actix_identity::Identity;
use ::actix_web::Error;
use ::actix_web::Responder;
use ::actix_web::error::ErrorInternalServerError;
use ::actix_web::error::ErrorUnauthorized;
use ::actix_web::get;
use ::actix_web::web::Data;
use ::actix_web::web::Json;
use ::actix_web::web::Path;
use ::db::tiberius::create_client;
use ::db::timekeeper::TimeStamp;
use ::db::timekeeper::TimeStrip;
use ::tracing::error;

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
            error!(%err, "Failed to create DB client for timestrip");
            ErrorInternalServerError(err)
        })?;
        let timestrip = TimeStrip::load(client).await.map_err(|err| {
            error!(%err, "Failed to load timestrip data");
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
            error!(%err, regatta_id, "Failed to query statistics");
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
            error!(%err, regatta_id, "Failed to calculate scoring");
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
        error!(%err, regatta_id, "Failed to query schedule");
        ErrorInternalServerError(err)
    })?;
    Ok(Json(schedule))
}
