use mobc_redis::RedisConnectionManager;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    State,
};

use crate::services::oauth2_authorization_service::{AccessToken, ACCESS_TOKEN_KEY_REDIS_PREFIX};
use crate::{api::oauth::access_token::TOKEN_TYPE_BEARER, services::redis_service};

#[derive(Debug)]
pub struct AccessTokenAuth {
    pub access_token: AccessToken,
}

#[derive(Debug, thiserror::Error)]
pub enum AccessTokenError {
    #[error("Missing authorization header")]
    MissingAuthHeader,
    #[error("Invalid authorization header")]
    InvalidAuthHeader,
    #[error("Redis error")]
    RedisError,
}

const AUTH_HEADER_NAME: &str = "Authorization";

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AccessTokenAuth {
    type Error = AccessTokenError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let redis_pool = match request
            .guard::<&State<mobc::Pool<RedisConnectionManager>>>()
            .await
            .succeeded()
        {
            Some(pool) => pool,
            None => {
                error!("Failed to retrieve redis pool");
                return Outcome::Failure((
                    Status::InternalServerError,
                    AccessTokenError::RedisError,
                ));
            }
        };

        let auth_header = request.headers().get_one(AUTH_HEADER_NAME);
        let bearer_token = match auth_header {
            Some(s) => s.to_string(),
            None => {
                error!("Auth failed, missing bearer token header");
                return Outcome::Failure((
                    Status::Unauthorized,
                    AccessTokenError::MissingAuthHeader,
                ));
            }
        };

        let access_token = match bearer_token.strip_prefix(&format!("{TOKEN_TYPE_BEARER} ")) {
            Some(t) => t,
            None => {
                error!("Invalid bearer token '{bearer_token}'");
                return Outcome::Failure((
                    Status::Unauthorized,
                    AccessTokenError::InvalidAuthHeader,
                ));
            }
        };

        let key = format!("{}:{}", ACCESS_TOKEN_KEY_REDIS_PREFIX, access_token);
        let access_token: AccessToken =
            match redis_service::redis_get_option::<AccessToken>(redis_pool, key).await {
                Ok(Some(a)) => a,
                Ok(None) => {
                    println!("Invalid auth token {access_token}");
                    return Outcome::Failure((
                        Status::Unauthorized,
                        AccessTokenError::InvalidAuthHeader,
                    ));
                }
                Err(e) => {
                    error!("Failed to get access token from redis, err: {e}");
                    return Outcome::Failure((
                        Status::InternalServerError,
                        AccessTokenError::RedisError,
                    ));
                }
            };

        Outcome::Success(AccessTokenAuth { access_token })
    }
}
