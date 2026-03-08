use crate::db::UserPoolManager;
use crate::http::rest_api::INTERNAL_SERVER_ERROR;
use crate::http::rest_api::PATH;
use crate::http::rest_api::get_user_pool;
use ::actix_identity::Identity;
use ::actix_web::Error;
use ::actix_web::Responder;
use ::actix_web::error::ErrorInternalServerError;
use ::actix_web::error::ErrorUnauthorized;
use ::actix_web::get;
use ::actix_web::web::Data;
use ::actix_web::web::Json;
use ::actix_web::web::Path;
use ::db::aquarius::Aquarius;
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
async fn get_timestrip(
    identity: Option<Identity>,
    user_pool_manager: Data<UserPoolManager>,
) -> Result<impl Responder, Error> {
    if let Some(identity) = identity
        && let Ok(id) = identity.id()
        && id == "sa"
    {
        let pool = get_user_pool(&identity, &user_pool_manager).await?;
        let mut client = pool.get().await.map_err(|err| {
            error!(%err, "Failed to get DB client from pool");
            ErrorInternalServerError(err)
        })?;
        let timestrip = TimeStrip::load(&mut client).await.map_err(|err| {
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
    regatta_id: Path<i32>,
    aquarius: Data<Aquarius>,
    identity: Option<Identity>,
) -> Result<impl Responder, Error> {
    if identity.is_some() {
        let stats = aquarius
            .query_statistics(regatta_id.into_inner())
            .await
            .map_err(|err| {
                error!(%err, "Failed to query statistics");
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
    regatta_id: Path<i32>,
    aquarius: Data<Aquarius>,
    identity: Option<Identity>,
) -> Result<impl Responder, Error> {
    if identity.is_some() {
        let scoring = aquarius
            .calculate_scoring(regatta_id.into_inner())
            .await
            .map_err(|err| {
                error!(%err, "Failed to calculate scoring");
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
    regatta_id: Path<i32>,
    aquarius: Data<Aquarius>,
    identity: Option<Identity>,
) -> Result<impl Responder, Error> {
    let schedule = aquarius
        .query_schedule(regatta_id.into_inner(), identity.is_some())
        .await
        .map_err(|err| {
            error!(%err, "Failed to query schedule");
            ErrorInternalServerError(err)
        })?;
    Ok(Json(schedule))
}
