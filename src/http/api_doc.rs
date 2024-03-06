use crate::http::{
    auth::{Credentials, Scope, User},
    monitor::{Connections, Cpu, Db, Memory, Monitor, SysInfo},
    rest_api,
};
use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        rest_api::monitor, rest_api::identity, rest_api::login, rest_api::logout,
    ),
    components(
        schemas(Monitor, Db, Connections, User, Credentials, Scope, SysInfo, Cpu, Memory),
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

    cfg.service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()));
}
