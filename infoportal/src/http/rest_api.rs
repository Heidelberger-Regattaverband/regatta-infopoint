use crate::{
    db::aquarius::Aquarius,
    http::{
        auth::{Credentials, Scope as UserScope, User},
        ws,
    },
};
use actix_identity::Identity;
use actix_web::{
    Error, HttpMessage, HttpRequest, HttpResponse, Responder, Scope as ActixScope,
    error::{ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized, InternalError},
    get, post,
    web::{Data, Json, Path, ServiceConfig},
};
use db::{
    aquarius::model::{Athlete, Club, Entry, Filters, Heat, Race, Regatta},
    tiberius::TiberiusPool,
    timekeeper::{TimeStamp, TimeStrip},
};
use log::error;

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

// Athletes Endpoints

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
    let clubs = aquarius
        .get_participating_athletes(regatta_id, opt_user)
        .await
        .map_err(|err| {
            error!("{err}");
            ErrorInternalServerError(err)
        })?;
    Ok(Json(clubs))
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
    let clubs = aquarius
        .get_athlete(regatta_id, athlete_id, opt_user)
        .await
        .map_err(|err| {
            error!("{err}");
            ErrorInternalServerError(err)
        })?;
    Ok(Json(clubs))
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
    let ids = ids.into_inner();
    let entries = aquarius
        .get_athlete_entries(ids.0, ids.1, opt_user)
        .await
        .map_err(|err| {
            error!("{err}");
            ErrorInternalServerError(err)
        })?;
    Ok(Json(entries))
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
    if opt_user.is_some() {
        let timestrip = TimeStrip::load(TiberiusPool::instance()).await.map_err(|err| {
            error!("{err}");
            ErrorInternalServerError(err)
        })?;
        Ok(Json(timestrip.time_stamps))
    } else {
        Err(ErrorUnauthorized("Unauthorized"))
    }
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
            if let Err(e) = Identity::login(&request.extensions(), user.username.clone()) {
                log::error!("Failed to attach user identity to session: {}", e);
                return Err(ErrorInternalServerError("Failed to create session"));
            }
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
        match user.id() {
            Ok(id) => Ok(Json(User::new(id, UserScope::User))),
            Err(e) => {
                log::error!("Failed to get user ID from identity: {}", e);
                Err(ErrorInternalServerError("Failed to get user identity"))
            }
        }
    } else {
        Err(InternalError::from_response("", HttpResponse::Unauthorized().json(User::new_guest())).into())
    }
}

/// Configure the REST API. This will add all REST API endpoints to the service configuration.
pub(crate) fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        ActixScope::new(PATH)
            .service(get_athlete)
            .service(get_regatta_club)
            .service(get_club_entries)
            .service(get_athlete_entries)
            .service(get_participating_clubs)
            .service(get_participating_athletes)
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
            .service(login)
            .service(identity)
            .service(logout)
            .service(ws::index),
    );
}
