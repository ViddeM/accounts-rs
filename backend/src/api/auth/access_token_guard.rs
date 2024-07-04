use mobc_redis::RedisConnectionManager;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    serde::json::Json,
    State,
};

use crate::{
    api::oauth::access_token::AccessTokenError,
    services::oauth_authorization_service::{AccessToken, ACCESS_TOKEN_KEY_REDIS_PREFIX},
};
use crate::{api::oauth::access_token::TOKEN_TYPE_BEARER, services::redis_service};

#[derive(Debug)]
pub struct AccessTokenAuth {
    pub access_token: AccessToken,
}

#[derive(Debug, Clone, Responder)]
pub enum AccessTokenAuthError {
    #[response(status = 400)]
    AuthError(Json<AccessTokenError>),
    #[response(status = 500)]
    InternalError(String),
}

const AUTH_HEADER_NAME: &str = "Authorization";

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AccessTokenAuth {
    type Error = AccessTokenAuthError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let redis_pool = match request
            .guard::<&State<mobc::Pool<RedisConnectionManager>>>()
            .await
            .succeeded()
        {
            Some(pool) => pool,
            None => {
                error!("Failed to retrieve redis pool");
                return Outcome::Error((
                    Status::InternalServerError,
                    AccessTokenAuthError::InternalError("Internal error".to_string()),
                ));
            }
        };

        let auth_header = request.headers().get_one(AUTH_HEADER_NAME);
        let bearer_token = match auth_header {
            Some(s) => s.to_string(),
            None => {
                error!("Auth failed, missing bearer token header");
                return Outcome::Error((
                    Status::BadRequest,
                    AccessTokenAuthError::AuthError(Json(AccessTokenError::InvalidRequest)),
                ));
            }
        };

        let access_token = match bearer_token.strip_prefix(&format!("{TOKEN_TYPE_BEARER} ")) {
            Some(t) => t,
            None => {
                error!("Invalid bearer token '{bearer_token}'");
                return Outcome::Error((
                    Status::BadRequest,
                    AccessTokenAuthError::AuthError(Json(AccessTokenError::InvalidRequest)),
                ));
            }
        };

        let key = format!("{}:{}", ACCESS_TOKEN_KEY_REDIS_PREFIX, access_token);
        let access_token: AccessToken =
            match redis_service::redis_get_option::<AccessToken>(redis_pool, key).await {
                Ok(Some(a)) => a,
                Ok(None) => {
                    println!("Invalid auth token {access_token}");
                    return Outcome::Error((
                        Status::BadRequest,
                        AccessTokenAuthError::AuthError(Json(AccessTokenError::InvalidRequest)),
                    ));
                }
                Err(e) => {
                    error!("Failed to get access token from redis, err: {e}");
                    return Outcome::Error((
                        Status::InternalServerError,
                        AccessTokenAuthError::InternalError("Internal error".to_string()),
                    ));
                }
            };

        Outcome::Success(AccessTokenAuth { access_token })
    }
}
