use crate::http::{
    auth::{Credentials, Scope, User},
    monitoring::{Connections, Cpu, Db, Disk, Memory, Monitoring, SysInfo},
    rest_api,
};
use actix_web::web;
use db::{
    aquarius::model::{
        AgeClass, Athlete, BoatClass, Club, Crew, Entry, Filters, Heat, HeatEntry, Race, Referee, Regatta,
    },
    timekeeper::TimeStamp,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        rest_api::identity,
        rest_api::login,
        rest_api::logout,
        rest_api::get_filters,
        rest_api::get_active_regatta,
        rest_api::get_races,
        rest_api::get_race,
        rest_api::get_heats,
        rest_api::get_heat,
        rest_api::get_participating_clubs,
        rest_api::get_club_entries,
        rest_api::get_regatta_club,
        rest_api::get_participating_athletes,
        rest_api::get_athlete,
        rest_api::get_athlete_entries,
        rest_api::get_timestrip,
        rest_api::get_statistics,
        rest_api::calculate_scoring,
        rest_api::get_schedule
    ),
    components(
        schemas(
            Monitoring, Db, Connections, User, Credentials, Scope, SysInfo, Cpu, Memory, Disk,
            Regatta, AgeClass, BoatClass, Filters, Club, Entry, Athlete, Crew, Heat, HeatEntry, Referee, Race, TimeStamp
        ),
    ),
    tags(
        (name = "regatta-infopoint", description = "Regatta Infopoint endpoints.")
    )
)]
struct ApiDoc;

/// Configure the API documentation.
pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    // Make instance variable of ApiDoc so all worker threads gets the same instance.
    let openapi = ApiDoc::openapi();

    cfg.service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi));
}
