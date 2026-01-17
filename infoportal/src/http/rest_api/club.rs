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
use ::db::aquarius::model::Club;
use ::db::aquarius::model::Entry;
use ::tracing::error;

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
            error!(%err, regatta_id, "Failed to get participating clubs");
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
    let (regatta_id, club_id) = ids.into_inner();
    let entries = aquarius
        .get_club_entries(regatta_id, club_id, opt_user)
        .await
        .map_err(|err| {
            error!(%err, regatta_id, club_id,"Failed to get club entries");
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
    let (regatta_id, club_id) = ids.into_inner();
    let club = aquarius
        .get_regatta_club(regatta_id, club_id, opt_user)
        .await
        .map_err(|err| {
            error!(%err, regatta_id, club_id, "Failed to get club");
            ErrorInternalServerError(err)
        })?;
    Ok(Json(club))
}
