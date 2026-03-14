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
use ::db::aquarius::Aquarius;
use ::db::aquarius::model::Race;
use ::tracing::error;

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
    regatta_id: Path<i32>,
    aquarius: Data<Aquarius>,
    identity: Option<Identity>,
) -> Result<impl Responder, Error> {
    let races = aquarius
        .get_races(regatta_id.into_inner(), identity.is_some())
        .await
        .map_err(|err| {
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
    race_id: Path<i32>,
    aquarius: Data<Aquarius>,
    identity: Option<Identity>,
) -> Result<impl Responder, Error> {
    let race = aquarius
        .get_race_heats_entries(race_id.into_inner(), identity.is_some())
        .await
        .map_err(|err| {
            error!("{err}");
            ErrorInternalServerError(err)
        })?;
    Ok(Json(race))
}
