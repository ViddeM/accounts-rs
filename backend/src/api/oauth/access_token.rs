use chrono::Utc;
use mobc_redis::RedisConnectionManager;
use rocket::{form::Form, http::Status, serde::json::Json, Response, State};
use serde::Serialize;
use sqlx::Pool;

use crate::{
    db::DB,
    services::oauth2_authorization_service::{self, Oauth2Error},
};
use rocket::response::Responder;

const GRANT_TYPE_AUTHORIZATION_CODE: &str = "authorization_code";
const HEADER_CACHE_CONTROL: &str = "Cache-Control";
const HEADER_PRAGMA: &str = "Pragma";

const NO_CACHE: &str = "no-cache";
const NO_STORE: &str = "no-store";

pub const TOKEN_TYPE_BEARER: &str = "Bearer";

#[derive(FromForm, Debug)]
pub struct AccessTokenRequest {
    grant_type: String,
    redirect_uri: String,
    code: String,
    client_id: String,
    client_secret: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct AccessTokenResponse {
    access_token: String,
    expires_in: u32,
    token_type: String,
}

pub enum PostAccessTokenResponse {
    Success(Json<AccessTokenResponse>),
    Error(String, Status),
}

#[derive(Serialize, Clone, Debug)]
pub struct AccessTokenErrorResponse {
    message: String,
}

// Second step in the oauth2 authorization flow.
#[post("/token", data = "<request>")]
pub async fn post_access_token(
    db_pool: &State<Pool<DB>>,
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
    request: Form<AccessTokenRequest>,
) -> PostAccessTokenResponse {
    if request.grant_type != GRANT_TYPE_AUTHORIZATION_CODE {
        return PostAccessTokenResponse::Error(
            String::from("Invalid grant type"),
            Status::UnprocessableEntity,
        );
    }

    let access_token = match oauth2_authorization_service::get_access_token(
        db_pool,
        redis_pool,
        request.client_id.clone(),
        request.client_secret.clone(),
        request.redirect_uri.clone(),
        request.code.clone(),
    )
    .await
    {
        Ok(access_token) => access_token,
        Err(Oauth2Error::NoClientWithId) => {
            return PostAccessTokenResponse::Error(
                String::from("Invalid client ID"),
                Status::BadRequest,
            );
        }
        Err(Oauth2Error::InvalidRedirectUri) => {
            return PostAccessTokenResponse::Error(
                String::from("Invalid redirect URI"),
                Status::BadRequest,
            );
        }
        Err(Oauth2Error::InvalidClientSecret) => {
            return PostAccessTokenResponse::Error(
                String::from("Invalid client secret"),
                Status::BadRequest,
            );
        }
        Err(Oauth2Error::InvalidCode) => {
            return PostAccessTokenResponse::Error(
                String::from("Invalid code"),
                Status::BadRequest,
            );
        }
        Err(err) => {
            error!("Failed to get access token, err: {}", err);
            return PostAccessTokenResponse::Error(
                String::from("An internal server error occurred"),
                Status::BadRequest,
            );
        }
    };

    let now = Utc::now();
    let expires_in = access_token.expiration.timestamp() - now.timestamp(); // The number of seconds until expiration
    if expires_in <= 0 {
        warn!("Expires in is {expires_in} before being returned to the caller!");
    }
    let expires_in = expires_in as u32; // Just checked that it's not negative so this is safe.

    let access_token_response = AccessTokenResponse {
        access_token: access_token.access_token,
        expires_in,
        token_type: TOKEN_TYPE_BEARER.to_string(),
    };

    PostAccessTokenResponse::Success(Json(access_token_response))
}

impl<'r> Responder<'r, 'static> for PostAccessTokenResponse {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        match self {
            PostAccessTokenResponse::Success(content) => {
                let mut response = Response::build_from(content.respond_to(request)?);
                response.status(Status::Ok);
                response.raw_header(HEADER_CACHE_CONTROL, NO_STORE);
                response.raw_header(HEADER_PRAGMA, NO_CACHE);
                response.ok()
            }
            PostAccessTokenResponse::Error(msg, status) => {
                let err_response = Json(AccessTokenErrorResponse { message: msg });
                let mut response = Response::build_from(err_response.respond_to(request)?);
                response.status(status);
                response.ok()
            }
        }
    }
}
