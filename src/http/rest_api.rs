use crate::{
    db::{
        aquarius::Aquarius,
        model::{Club, Filters, Heat, Race, Regatta},
        tiberius::TiberiusPool,
    },
    http::{
        auth::{Credentials, Scope, User},
        monitoring::Monitoring,
    },
};
use actix_identity::Identity;
use actix_web::{
    error::{ErrorUnauthorized, InternalError},
    get, post,
    web::{self, Data, Json, Path},
    Error, HttpMessage, HttpRequest, HttpResponse, Responder,
};
use prometheus::Registry;

use super::ws;

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
async fn get_regattas(aquarius: Data<Aquarius>) -> Json<Vec<Regatta>> {
    Json(aquarius.query_regattas().await)
}

#[get("/active_regatta")]
async fn get_active_regatta(aquarius: Data<Aquarius>, opt_user: Option<Identity>) -> Json<Regatta> {
    Json(aquarius.get_active_regatta(opt_user).await)
}

#[get("/regattas/{id}")]
async fn get_regatta(path: Path<i32>, aquarius: Data<Aquarius>, opt_user: Option<Identity>) -> Json<Regatta> {
    let regatta_id = path.into_inner();
    Json(aquarius.get_regatta(regatta_id, opt_user).await)
}

#[get("/regattas/{id}/races")]
async fn get_races(path: Path<i32>, aquarius: Data<Aquarius>, opt_user: Option<Identity>) -> Json<Vec<Race>> {
    let regatta_id = path.into_inner();
    Json(aquarius.get_races(regatta_id, opt_user).await)
}

#[get("/races/{id}")]
async fn get_race(path: Path<i32>, aquarius: Data<Aquarius>, opt_user: Option<Identity>) -> Json<Race> {
    let race_id = path.into_inner();
    Json(aquarius.get_race(race_id, opt_user).await)
}

#[get("/regattas/{id}/heats")]
async fn get_heats(path: Path<i32>, aquarius: Data<Aquarius>, opt_user: Option<Identity>) -> Json<Vec<Heat>> {
    let regatta_id = path.into_inner();
    Json(aquarius.get_heats(regatta_id, opt_user).await)
}

#[get("/regattas/{id}/filters")]
async fn get_filters(path: Path<i32>, aquarius: Data<Aquarius>, opt_user: Option<Identity>) -> Json<Filters> {
    let regatta_id = path.into_inner();
    Json(aquarius.get_filters(regatta_id, opt_user).await)
}

#[get("/heats/{id}")]
async fn get_heat(path: Path<i32>, aquarius: Data<Aquarius>, opt_user: Option<Identity>) -> Json<Heat> {
    let heat_id = path.into_inner();
    Json(aquarius.get_heat(heat_id, opt_user).await)
}

#[get("/regattas/{id}/participating_clubs")]
async fn get_participating_clubs(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> impl Responder {
    let regatta_id = path.into_inner();
    Json(aquarius.get_participating_clubs(regatta_id, opt_user).await)
}

#[get("/regattas/{regatta_id}/clubs/{club_id}/heats")]
async fn get_club_heats(ids: Path<(i32, i32)>, aquarius: Data<Aquarius>, opt_user: Option<Identity>) -> impl Responder {
    let ids = ids.into_inner();
    Json(aquarius.get_club_heats(ids.0, ids.1, opt_user).await)
}

#[get("/regattas/{regatta_id}/clubs/{club_id}/registrations")]
async fn get_club_registrations(
    ids: Path<(i32, i32)>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> impl Responder {
    let ids = ids.into_inner();
    Json(aquarius.get_club_registrations(ids.0, ids.1, opt_user).await)
}

#[get("/clubs/{id}")]
async fn get_club(path: Path<i32>, aquarius: Data<Aquarius>) -> Json<Club> {
    let club_id = path.into_inner();
    Json(aquarius.get_club(club_id).await)
}

#[get("/regattas/{id}/statistics")]
async fn get_statistics(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    if opt_user.is_some() {
        let regatta_id = path.into_inner();
        Ok(Json(aquarius.query_statistics(regatta_id).await))
    } else {
        Err(ErrorUnauthorized("Unauthorized"))
    }
}

#[get("/regattas/{id}/kiosk")]
async fn get_kiosk(
    path: Path<i32>,
    aquarius: Data<Aquarius>,
    opt_user: Option<Identity>,
) -> Result<impl Responder, Error> {
    if opt_user.is_some() {
        let regatta_id = path.into_inner();
        Ok(Json(aquarius.query_kiosk(regatta_id).await))
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
        Ok(Json(aquarius.calculate_scoring(regatta_id).await))
    } else {
        Err(ErrorUnauthorized("Unauthorized"))
    }
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
        Ok(Json(User::new(user.id().unwrap(), Scope::User)))
    } else {
        Err(InternalError::from_response("", HttpResponse::Unauthorized().json(User::new_guest())).into())
    }
}

/// Configure the REST API. This will add all REST API endpoints to the service configuration.
pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(PATH)
            .service(get_club)
            .service(get_regattas)
            .service(get_club_heats)
            .service(get_club_registrations)
            .service(get_participating_clubs)
            .service(get_active_regatta)
            .service(get_regatta)
            .service(get_race)
            .service(get_races)
            .service(get_heats)
            .service(get_filters)
            .service(get_heat)
            .service(get_kiosk)
            .service(calculate_scoring)
            .service(get_statistics)
            .service(login)
            .service(identity)
            .service(logout)
            .service(monitoring)
            .service(ws::index),
    );
}
