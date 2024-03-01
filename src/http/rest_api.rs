use crate::{
    db::{
        aquarius::Aquarius,
        model::{Club, Filters, Heat, Race, Regatta},
    },
    http::{
        auth::{Credentials, Scope, User},
        monitor::Monitor,
    },
};
use actix_identity::Identity;
use actix_web::{
    error::{ErrorUnauthorized, InternalError},
    get, post,
    web::{Data, Json, Path},
    Error, HttpMessage, HttpRequest, HttpResponse, Responder,
};

#[utoipa::path(
    responses(
        (status = 200, description = "Monitoring", body = Monitor)
    )
)]
#[get("/monitor")]
async fn monitor(aquarius: Data<Aquarius>) -> Json<Monitor> {
    let pool = aquarius.pool.state();
    Json(Monitor::new(pool, aquarius.pool.created()))
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

#[post("/logout")]
async fn logout(user: Identity) -> impl Responder {
    user.logout();
    HttpResponse::NoContent()
}

#[get("/identity")]
async fn identity(opt_user: Option<Identity>) -> Result<impl Responder, Error> {
    if let Some(user) = opt_user {
        Ok(Json(User::new(user.id().unwrap(), Scope::User)))
    } else {
        Err(InternalError::from_response("", HttpResponse::Unauthorized().json(User::new_guest())).into())
    }
}
