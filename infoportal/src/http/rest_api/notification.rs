use crate::db::aquarius::Aquarius;
use crate::http::rest_api::INTERNAL_SERVER_ERROR;
use crate::http::rest_api::PATH;
use ::actix_session::Session;
use ::actix_web::Error;
use ::actix_web::HttpResponse;
use ::actix_web::Responder;
use ::actix_web::error::ErrorInternalServerError;
use ::actix_web::get;
use ::actix_web::post;
use ::actix_web::web::Data;
use ::actix_web::web::Json;
use ::actix_web::web::Path;
use ::db::aquarius::model::Notification;
use ::tiberius::time::chrono::DateTime;
use ::tiberius::time::chrono::Utc;
use ::tracing::debug;
use ::tracing::error;

#[utoipa::path(
    description = "Get all notifcations for a regatta.",
    context_path = PATH,
    responses(
        (status = 200, description = "Notifications for <regatta_id>", body = Vec<Notification>),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/{regatta_id}/notifications")]
async fn get_notifications(
    regatta_id: Path<i32>,
    aquarius: Data<Aquarius>,
    session: Session,
) -> Result<impl Responder, Error> {
    session.entries().iter().for_each(|(key, value)| {
        debug!(key, value, "Query Notification Session Entry");
    });
    let regatta_id = regatta_id.into_inner();
    let all_notifications = aquarius.get_notifications(regatta_id).await.map_err(|err| {
        error!(%err, regatta_id, "Failed to get notifications");
        ErrorInternalServerError(err)
    })?;

    let notifications: Vec<Notification> = all_notifications
        .into_iter()
        .filter(|notification| {
            let read_value = session
                .get::<DateTime<Utc>>(&create_notification_read_key(notification.id))
                .unwrap_or(None);
            let read = read_value.is_some_and(|read| read > notification.modified_at);
            !read
        })
        .collect();
    Ok(Json(notifications))
}

#[post("/notifications/{notification_id}/read")]
async fn notification_read(notification_id: Path<i32>, session: Session) -> Result<impl Responder, Error> {
    session.insert(create_notification_read_key(notification_id.into_inner()), Utc::now())?;
    session.renew();
    Ok(HttpResponse::NoContent())
}

fn create_notification_read_key(notification_id: i32) -> String {
    format!("notifications.{}.read", notification_id)
}
