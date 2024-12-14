use crate::db::aquarius::Aquarius;
use actix_web::{
    get,
    http::Error,
    web::{scope, Data, Json, ServiceConfig},
    Responder,
};

/// Path to reports endpoint
pub(crate) const PATH: &str = "/reports";

#[get("/regattas")]
async fn get_regattas(aquarius: Data<Aquarius>) -> Result<impl Responder, Error> {
    Ok(Json(aquarius.query_regattas().await))
}

/// Configure the REST API. This will add all REST API endpoints to the service configuration.
pub(crate) fn config(cfg: &mut ServiceConfig) {
    cfg.service(scope(PATH).service(get_regattas));
}
