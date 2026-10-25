use crate::auth::Credentials;
use crate::auth::Scope;
use crate::auth::User;
use crate::http::rest_api::ApiError;
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
use ::actix_web::web::Data;
use ::actix_web::web::Json;
use ::db::tiberius::user_pool::UserPoolManager;
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
async fn login(
    credentials: Json<Credentials>,
    request: HttpRequest,
    user_pool_manager: Data<UserPoolManager>,
) -> Result<impl Responder, Error> {
    let credentials = credentials.into_inner();
    // authenticates the user, without creating a pool.
    match User::authenticate(&credentials).await {
        // authentication succeeded
        Ok(user) => {
            // create DB pool first — if this fails, no session cookie is set
            user_pool_manager
                .create_pool(&user.username, credentials.password.value())
                .await
                .map_err(ApiError::from)?;
            // attach valid user identity to current session
            let login_result = Identity::login(&request.extensions(), user.username.clone());
            if let Err(err) = login_result {
                error!(%err, user = user.username, "Failed to attach user identity to session");
                user_pool_manager.remove_pool(&user.username).await;
                return Err(ErrorInternalServerError("Failed to create session"));
            }
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
async fn logout(ident: Identity, user_pool_manager: Data<UserPoolManager>) -> impl Responder {
    if let Ok(username) = ident.id() {
        user_pool_manager.remove_pool(&username).await;
    }
    ident.logout();
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
                let scope = Scope::from_username(&id);
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
