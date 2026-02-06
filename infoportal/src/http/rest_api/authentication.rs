use crate::http::auth::Credentials;
use crate::http::auth::Scope as UserScope;
use crate::http::auth::User;
use crate::http::rest_api::PATH;
use ::actix_identity::Identity;
use ::actix_web::Error;
use ::actix_web::HttpMessage;
use ::actix_web::HttpRequest;
use ::actix_web::HttpResponse;
use ::actix_web::Responder;
use ::actix_web::error::ErrorInternalServerError;
use ::actix_web::error::InternalError;
use ::actix_web::get;
use ::actix_web::post;
use ::actix_web::web::Json;
use ::tracing::error;

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
            if let Err(err) = Identity::login(&request.extensions(), user.username.clone()) {
                error!(%err, user = user.username, "Failed to attach user identity to session");
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
async fn identity(identity: Option<Identity>) -> Result<impl Responder, Error> {
    if let Some(identity) = identity {
        match identity.id() {
            Ok(id) => {
                let scope = match id.as_str() {
                    "sa" | "admin" => UserScope::Admin,
                    _ => UserScope::User,
                };
                Ok(Json(User::new(id, scope)))
            }
            Err(err) => {
                error!(%err, "Failed to get identity from session");
                Err(ErrorInternalServerError("Failed to get identity"))
            }
        }
    } else {
        Err(InternalError::from_response("", HttpResponse::Unauthorized().json(User::new_guest())).into())
    }
}
