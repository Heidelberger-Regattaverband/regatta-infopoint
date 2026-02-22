use crate::db::aquarius::Aquarius;
use crate::http::rest_api::INTERNAL_SERVER_ERROR;
use crate::http::rest_api::PATH;
use ::actix_identity::Identity;
use ::actix_session::Session;
use ::actix_web::Error;
use ::actix_web::HttpResponse;
use ::actix_web::Responder;
use ::actix_web::delete;
use ::actix_web::error::ErrorInternalServerError;
use ::actix_web::error::ErrorUnauthorized;
use ::actix_web::get;
use ::actix_web::post;
use ::actix_web::put;
use ::actix_web::web::Data;
use ::actix_web::web::Json;
use ::actix_web::web::Path;
use ::db::aquarius::model::{CreateNotificationRequest, Notification, UpdateNotificationRequest};
use ::serde_json::json;
use ::tiberius::time::chrono::DateTime;
use ::tiberius::time::chrono::Utc;
use ::tracing::error;

#[utoipa::path(
    description = "Get visible notifications for a regatta.",
    context_path = PATH,
    responses(
        (status = 200, description = "Notifications for <regatta_id>", body = Vec<Notification>),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/{regatta_id}/visible_notifications")]
async fn get_visible_notifications(
    regatta_id: Path<i32>,
    aquarius: Data<Aquarius>,
    session: Session,
) -> Result<impl Responder, Error> {
    let regatta_id = regatta_id.into_inner();
    let visible_notifications = aquarius.get_visible_notifications(regatta_id).await.map_err(|err| {
        error!(%err, regatta_id, "Failed to get visible notifications");
        ErrorInternalServerError(err)
    })?;

    let notifications: Vec<Notification> = visible_notifications
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

#[utoipa::path(
    description = "Get all notifications for a regatta (admin only - includes invisible notifications).",
    context_path = PATH,
    responses(
        (status = 200, description = "All notifications for <regatta_id>", body = Vec<Notification>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/{regatta_id}/notifications")]
async fn get_all_notifications(
    regatta_id: Path<i32>,
    aquarius: Data<Aquarius>,
    identity: Option<Identity>,
) -> Result<impl Responder, Error> {
    if identity.is_none() {
        return Err(ErrorUnauthorized("Unauthorized"));
    }

    let regatta_id = regatta_id.into_inner();
    let all_notifications = aquarius.get_all_notifications(regatta_id).await.map_err(|err| {
        error!(%err, regatta_id, "Failed to get all notifications");
        ErrorInternalServerError(err)
    })?;

    Ok(Json(all_notifications))
}

#[utoipa::path(
    description = "Create a new notification for a regatta.",
    context_path = PATH,
    request_body = CreateNotificationRequest,
    responses(
        (status = 201, description = "Notification created successfully", body = Notification),
        (status = 400, description = "Invalid request body"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[post("/regattas/{regatta_id}/notifications")]
async fn create_notification(
    regatta_id: Path<i32>,
    request: Json<CreateNotificationRequest>,
    aquarius: Data<Aquarius>,
    identity: Option<Identity>,
) -> Result<impl Responder, Error> {
    if identity.is_none() {
        return Err(ErrorUnauthorized("Unauthorized"));
    }

    let regatta_id = regatta_id.into_inner();
    let request = request.into_inner();

    // Basic validation
    if request.title.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Title cannot be empty"
        })));
    }

    let notification = aquarius
        .create_notification(regatta_id, &request)
        .await
        .map_err(|err| {
            error!(%err, regatta_id, "Failed to create notification");
            ErrorInternalServerError(err)
        })?;
    Ok(HttpResponse::Created().json(notification))
}

#[utoipa::path(
    description = "Update an existing notification.",
    context_path = PATH,
    request_body = UpdateNotificationRequest,
    responses(
        (status = 200, description = "Notification updated successfully", body = Notification),
        (status = 400, description = "Invalid request body"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Notification not found"),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[put("/notifications/{notification_id}")]
async fn update_notification(
    notification_id: Path<i32>,
    request: Json<UpdateNotificationRequest>,
    aquarius: Data<Aquarius>,
    identity: Option<Identity>,
) -> Result<impl Responder, Error> {
    if identity.is_none() {
        return Err(ErrorUnauthorized("Unauthorized"));
    }

    let notification_id = notification_id.into_inner();
    let request = request.into_inner();

    // Basic validation
    if let Some(ref title) = request.title
        && title.trim().is_empty()
    {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Title cannot be empty"
        })));
    }

    let notification = aquarius
        .update_notification(notification_id, &request)
        .await
        .map_err(|err| {
            error!(%err, notification_id, "Failed to update notification");
            ErrorInternalServerError(err)
        })?;

    match notification {
        Some(notification) => Ok(HttpResponse::Ok().json(notification)),
        None => Ok(HttpResponse::NotFound().json(json!({
            "error": "Notification not found"
        }))),
    }
}

#[utoipa::path(
    description = "Delete a notification by ID.",
    context_path = PATH,
    responses(
        (status = 204, description = "Notification deleted successfully"),
        (status = 404, description = "Notification not found"),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[delete("/notifications/{notification_id}")]
async fn delete_notification(
    notification_id: Path<i32>,
    aquarius: Data<Aquarius>,
    identity: Option<Identity>,
) -> Result<impl Responder, Error> {
    if identity.is_none() {
        return Err(ErrorUnauthorized("Unauthorized"));
    }

    let notification_id = notification_id.into_inner();

    let deleted = aquarius.delete_notification(notification_id).await.map_err(|err| {
        error!(%err, notification_id, "Failed to delete notification");
        ErrorInternalServerError(err)
    })?;

    if deleted {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::NotFound().json(json!({
            "error": "Notification not found"
        })))
    }
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
