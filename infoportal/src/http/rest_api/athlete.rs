// Athletes Endpoints

use crate::db::aquarius::Aquarius;
use crate::http::rest_api::INTERNAL_SERVER_ERROR;
use crate::http::rest_api::PATH;
use ::actix_identity::Identity;
use ::actix_web::Error;
use ::actix_web::Responder;
use ::actix_web::error::ErrorInternalServerError;
use ::actix_web::get;
use ::actix_web::web::Data;
use ::actix_web::web::Json;
use ::actix_web::web::Path;
use ::db::aquarius::model::Athlete;
use ::db::aquarius::model::Entry;
use ::tracing::error;

#[utoipa::path(
    description = "Get all participating athletes of a regatta.",
    context_path = PATH,
    responses(
        (status = 200, description = "Participating athletes", body = Vec<Athlete>),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/{regatta_id}/athletes")]
async fn get_participating_athletes(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let regatta_id = path.into_inner();
    let athletes = aquarius
        .get_participating_athletes(regatta_id, opt_user)
        .await
        .map_err(|err| {
            error!(%err, regatta_id, "Failed to get participating athletes");
            ErrorInternalServerError(err)
        })?;
    Ok(Json(athletes))
}

#[utoipa::path(
    description = "Get a specific athlete participating in a regatta.",
    context_path = PATH,
    responses(
        (status = 200, description = "Athlete details", body = Athlete),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/{regatta_id}/athletes/{athlete_id}")]
async fn get_athlete(
    path: Path<(i32, i32)>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let (regatta_id, athlete_id) = path.into_inner();
    let athletes = aquarius
        .get_athlete(regatta_id, athlete_id, opt_user)
        .await
        .map_err(|err| {
            error!(%err, regatta_id, athlete_id, "Failed to get athlete details");
            ErrorInternalServerError(err)
        })?;
    Ok(Json(athletes))
}

#[utoipa::path(
    description = "Get all entries of a specific athlete in a regatta.",
    context_path = PATH,
    responses(
        (status = 200, description = "Athlete entries", body = Vec<Entry>),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/{regatta_id}/athletes/{athlete_id}/entries")]
async fn get_athlete_entries(
    ids: Path<(i32, i32)>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let (regatta_id, athlete_id) = ids.into_inner();
    let entries = aquarius
        .get_athlete_entries(regatta_id, athlete_id, opt_user)
        .await
        .map_err(|err| {
            error!(%err, regatta_id, athlete_id, "Failed to get athlete entries");
            ErrorInternalServerError(err)
        })?;
    Ok(Json(entries))
}
