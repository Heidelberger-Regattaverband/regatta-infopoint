use crate::http::{
    auth::{Credentials, Scope, User},
    monitoring::{Connections, Cpu, Db, Disk, Memory, Monitoring, SysInfo},
    rest_api,
};
use ::db::aquarius::model::Notification;
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
        rest_api::authentication::identity,
        rest_api::authentication::login,
        rest_api::authentication::logout,
        rest_api::get_filters,
        rest_api::get_active_regatta,
        rest_api::get_races,
        rest_api::get_race,
        rest_api::get_heats,
        rest_api::get_heat,
        rest_api::club::get_participating_clubs,
        rest_api::club::get_club_entries,
        rest_api::club::get_regatta_club,
        rest_api::athlete::get_participating_athletes,
        rest_api::athlete::get_athlete,
        rest_api::athlete::get_athlete_entries,
        rest_api::misc::get_timestrip,
        rest_api::misc::get_statistics,
        rest_api::misc::calculate_scoring,
        rest_api::misc::get_schedule,
        rest_api::notification::get_notifications
    ),
    components(
        schemas(
            Monitoring, Db, Connections, User, Credentials, Scope, SysInfo, Cpu, Memory, Disk, Notification,
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
