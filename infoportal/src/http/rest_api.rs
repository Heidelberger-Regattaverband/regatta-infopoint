use crate::{
    db::aquarius::Aquarius,
    http::{
        auth::{Credentials, Scope as UserScope, User},
        monitoring::Monitoring,
        ws,
    },
};
use actix_identity::Identity;
use actix_web::{
    Error, HttpMessage, HttpRequest, HttpResponse, Responder, Scope as ActixScope,
    error::{ErrorInternalServerError, ErrorUnauthorized, InternalError},
    get, post,
    web::{Data, Json, Path, ServiceConfig},
};
use aquarius::db::tiberius::TiberiusPool;
use prometheus::Registry;

/// Path to REST API
pub(crate) const PATH: &str = "/api";

/// Provides the monitoring information.
#[utoipa::path(
    context_path = PATH,
    responses(
        (status = 200, description = "Monitoring", body = Monitoring),
        (status = 401, description = "Unauthorized")
    )
)]
#[get("/monitoring2")]
async fn monitoring(registry: Data<Registry>, opt_user: Option<Identity>) -> Result<impl Responder, Error> {
    if opt_user.is_some() {
        let monitoring = Monitoring::new(TiberiusPool::instance(), &registry);
        Ok(Json(monitoring))
    } else {
        Err(ErrorUnauthorized("Unauthorized"))
    }
}

#[get("/regattas")]
async fn get_regattas(aquarius: Data<Aquarius>) -> Result<impl Responder, Error> {
    let regattas = aquarius
        .query_regattas()
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(regattas))
}

#[get("/active_regatta")]
async fn get_active_regatta(aquarius: Data<Aquarius>, opt_user: Option<Identity>) -> Result<impl Responder, Error> {
    let regatta = aquarius
        .get_active_regatta(opt_user)
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(regatta))
}

#[get("/regattas/{id}")]
async fn get_regatta(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let regatta_id = path.into_inner();
    let regatta = aquarius
        .get_regatta(regatta_id, opt_user)
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(regatta))
}

#[get("/regattas/{id}/races")]
async fn get_races(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let regatta_id = path.into_inner();
    let races = aquarius
        .get_races(regatta_id, opt_user)
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(races))
}

#[get("/races/{id}")]
async fn get_race(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let race_id = path.into_inner();
    let race = aquarius
        .get_race_heats_registrations(race_id, opt_user)
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(race))
}

#[get("/regattas/{id}/heats")]
async fn get_heats(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let regatta_id = path.into_inner();
    let heats = aquarius
        .get_heats(regatta_id, opt_user)
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(heats))
}

#[get("/regattas/{id}/filters")]
async fn get_filters(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let regatta_id = path.into_inner();
    let filters = aquarius
        .get_filters(regatta_id, opt_user)
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(filters))
}

#[get("/heats/{id}")]
async fn get_heat(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let heat_id = path.into_inner();
    let heat = aquarius
        .get_heat(heat_id, opt_user)
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(heat))
}

#[get("/regattas/{id}/clubs")]
async fn get_participating_clubs(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let regatta_id = path.into_inner();
    let clubs = aquarius
        .get_participating_clubs(regatta_id, opt_user)
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(clubs))
}

#[get("/regattas/{id}/athletes")]
async fn get_participating_athletes(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let regatta_id = path.into_inner();
    let clubs = aquarius
        .get_participating_athletes(regatta_id, opt_user)
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(clubs))
}

#[get("/regattas/{regatta_id}/clubs/{club_id}/registrations")]
async fn get_club_registrations(
    ids: Path<(i32, i32)>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let ids = ids.into_inner();
    let registrations = aquarius
        .get_club_registrations(ids.0, ids.1, opt_user)
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(registrations))
}

#[get("/clubs/{club_id}")]
async fn get_club(path: Path<i32>, aquarius: Data<Aquarius>) -> Result<impl Responder, Error> {
    let club_id = path.into_inner();
    let club = aquarius
        .get_club(club_id)
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(club))
}

#[get("/athlete/{athlete_id}")]
async fn get_athlete(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let athlete_id = path.into_inner();
    let clubs = aquarius
        .get_athlete(athlete_id, opt_user)
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(clubs))
}

#[get("/regattas/{regatta_id}/athletes/{athlete_id}/registrations")]
async fn get_athlete_registrations(
    ids: Path<(i32, i32)>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    let ids = ids.into_inner();
    let registrations = aquarius
        .get_athlete_registrations(ids.0, ids.1, opt_user)
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(registrations))
}

#[get("/regattas/{regatta_id}/clubs/{club_id}")]
async fn get_regatta_club(ids: Path<(i32, i32)>, aquarius: Data<Aquarius>) -> Result<impl Responder, Error> {
    let ids = ids.into_inner();
    let club = aquarius
        .get_regatta_club(ids.0, ids.1)
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(club))
}

#[get("/regattas/{id}/statistics")]
async fn get_statistics(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    if opt_user.is_some() {
        let regatta_id = path.into_inner();
        let stats = aquarius
            .query_statistics(regatta_id)
            .await
            .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
        Ok(Json(stats))
    } else {
        Err(ErrorUnauthorized("Unauthorized"))
    }
}

#[get("/regattas/{id}/calculateScoring")]
async fn calculate_scoring(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    if opt_user.is_some() {
        let regatta_id = path.into_inner();
        let scoring = aquarius
            .calculate_scoring(regatta_id)
            .await
            .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
        Ok(Json(scoring))
    } else {
        Err(ErrorUnauthorized("Unauthorized"))
    }
}

#[get("/regattas/{id}/schedule")]
async fn get_schedule(path: Path<i32>, aquarius: Data<Aquarius>) -> Result<impl Responder, Error> {
    let regatta_id = path.into_inner();
    let schedule = aquarius
        .query_schedule(regatta_id)
        .await
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    Ok(Json(schedule))
}

/// Authenticate the user. This will attach the user identity to the current session.
#[utoipa::path(
    context_path = PATH,
    request_body = Credentials,
    responses(
        (status = 200, description = "Authenticated", body = User),
        (status = 401, description = "Unauthorized", body = User, example = json!({"user": "anonymous", "scope": "guest"}))
    )
)]
#[post("/login")]
async fn login(credentials: Json<Credentials>, request: HttpRequest) -> Result<impl Responder, Error> {
    match User::authenticate(credentials.into_inner()).await {
        // authentication succeeded
        Ok(user) => {
            // attach valid user identity to current session
            Identity::login(&request.extensions(), user.username.clone()).unwrap();
            // return user information: username and scope
            Ok(Json(user))
        }
        // authentication failed
        Err(err) => Err(InternalError::from_response("", err).into()),
    }
}

/// Logout the user. This will remove the user identity from the current session.
#[utoipa::path(
    context_path = PATH,
    responses(
        (status = 204, description = "User logged out"),
        (status = 401, description = "Unauthorized")
    )
)]
#[post("/logout")]
async fn logout(user: Identity) -> impl Responder {
    user.logout();
    HttpResponse::NoContent()
}

/// Get the user identity. This will return the user information if the user is authenticated. Otherwise, it will return a guest user.
#[utoipa::path(
    context_path = PATH,
    responses(
        (status = 200, description = "Authenticated", body = User, example = json!({"user": "name", "scope": "user"})),
        (status = 401, description = "Unauthorized", body = User, example = json!({ "user": "anonymous", "scope": "guest"}))
    )
)]
#[get("/identity")]
async fn identity(opt_user: Option<Identity>) -> Result<impl Responder, Error> {
    if let Some(user) = opt_user {
        Ok(Json(User::new(user.id().unwrap(), UserScope::User)))
    } else {
        Err(InternalError::from_response("", HttpResponse::Unauthorized().json(User::new_guest())).into())
    }
}

/// Configure the REST API. This will add all REST API endpoints to the service configuration.
pub(crate) fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        ActixScope::new(PATH)
            .service(get_club)
            .service(get_athlete)
            .service(get_regattas)
            .service(get_regatta_club)
            .service(get_club_registrations)
            .service(get_athlete_registrations)
            .service(get_participating_clubs)
            .service(get_participating_athletes)
            .service(get_active_regatta)
            .service(get_regatta)
            .service(get_race)
            .service(get_races)
            .service(get_heats)
            .service(get_filters)
            .service(get_heat)
            .service(calculate_scoring)
            .service(get_statistics)
            .service(get_schedule)
            .service(login)
            .service(identity)
            .service(logout)
            .service(monitoring)
            .service(ws::index),
    );
}
