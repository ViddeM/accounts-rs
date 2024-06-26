use std::str::from_utf8;

use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use mobc_redis::RedisConnectionManager;
use rocket::{
    form::Form,
    http::Status,
    request::{self, FromRequest},
    serde::json::Json,
    Request, Response, State,
};
use serde::Serialize;
use sqlx::Pool;

use crate::{
    db::DB,
    services::{
        login_service,
        oauth2_authorization_service::{self, AccessToken, Oauth2Error},
    },
    util::config::Config,
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
pub struct AccessTokenSuccessResponse {
    access_token: String,
    expires_in: u32,
    token_type: String,
}

pub enum AccessTokenResponse {
    Success(Json<AccessTokenSuccessResponse>),
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
) -> AccessTokenResponse {
    if request.grant_type != GRANT_TYPE_AUTHORIZATION_CODE {
        return AccessTokenResponse::Error(
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
            return AccessTokenResponse::Error(
                String::from("Invalid client ID"),
                Status::BadRequest,
            );
        }
        Err(Oauth2Error::InvalidRedirectUri) => {
            return AccessTokenResponse::Error(
                String::from("Invalid redirect URI"),
                Status::BadRequest,
            );
        }
        Err(Oauth2Error::InvalidClientSecret) => {
            return AccessTokenResponse::Error(
                String::from("Invalid client secret"),
                Status::BadRequest,
            );
        }
        Err(Oauth2Error::InvalidCode) => {
            return AccessTokenResponse::Error(String::from("Invalid code"), Status::BadRequest);
        }
        Err(err) => {
            error!("Failed to get access token, err: {}", err);
            return AccessTokenResponse::Error(
                String::from("An internal server error occurred"),
                Status::BadRequest,
            );
        }
    };

    let access_token_response = access_token.into();
    AccessTokenResponse::Success(Json(access_token_response))
}

impl<'r> Responder<'r, 'static> for AccessTokenResponse {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        match self {
            AccessTokenResponse::Success(content) => {
                let mut response = Response::build_from(content.respond_to(request)?);
                response.status(Status::Ok);
                response.raw_header(HEADER_CACHE_CONTROL, NO_STORE);
                response.raw_header(HEADER_PRAGMA, NO_CACHE);
                response.ok()
            }
            AccessTokenResponse::Error(msg, status) => {
                let err_response = Json(AccessTokenErrorResponse { message: msg });
                let mut response = Response::build_from(err_response.respond_to(request)?);
                response.status(status);
                response.ok()
            }
        }
    }
}

impl From<AccessToken> for AccessTokenSuccessResponse {
    fn from(value: AccessToken) -> Self {
        let now = Utc::now();
        let expires_in = value.expiration.timestamp() - now.timestamp(); // The number of seconds until expiration
        if expires_in <= 0 {
            warn!("Expires in is {expires_in} before being returned to the caller!");
        }
        let expires_in = expires_in as u32;

        AccessTokenSuccessResponse {
            access_token: value.access_token,
            expires_in,
            token_type: TOKEN_TYPE_BEARER.to_string(),
        }
    }
}

pub struct AuthHeader {
    username: String,
    password: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthHeader {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let Some(auth_header) = req.headers().get_one("authorization") else {
            error!("Got request with missing auth header");
            return request::Outcome::Error((
                Status::Unauthorized,
                "Missing auth header".to_string(),
            ));
        };

        let Some(auth_header) = auth_header.strip_prefix("Basic ") else {
            error!("Auth header missing 'Basic ' prefix, '{auth_header}'");
            return request::Outcome::Error((
                Status::Unauthorized,
                "Malformed auth header".to_string(),
            ));
        };

        let decoded = match STANDARD.decode(auth_header) {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to decode auth header {auth_header} due to err: {e:?}");
                return request::Outcome::Error((
                    Status::Unauthorized,
                    "Malformed auth header".to_string(),
                ));
            }
        };

        let decoded_string = match from_utf8(decoded.as_slice()) {
            Ok(v) => v,
            Err(e) => {
                error!(
                    "Auth header was not a valid utf8 string, header: {auth_header}, err: {e:?}"
                );

                return request::Outcome::Error((
                    Status::Unauthorized,
                    "Malformed auth header".to_string(),
                ));
            }
        };

        let Some((username, password)) = decoded_string.split_once(':') else {
            error!(
                "Failed to split username/password, invalid BASIC auth header, {decoded_string}"
            );
            return request::Outcome::Error((
                Status::Unauthorized,
                "Malformed auth header".to_string(),
            ));
        };

        request::Outcome::Success(AuthHeader {
            username: username.to_string(),
            password: password.to_string(),
        })
    }
}

// For docker flow
#[get("/token?<service>&<offline_token>&<client_id>&<scope>")]
#[allow(clippy::too_many_arguments)]
pub async fn get_access_token(
    db_pool: &State<sqlx::Pool<DB>>,
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
    config: &State<Config>,
    auth_header: AuthHeader,
    service: String,
    offline_token: Option<bool>,
    client_id: Option<String>,
    scope: Option<String>,
) -> AccessTokenResponse {
    info!(
        "Access token for user {}, service {service} and scopes {scope:?}",
        auth_header.username
    );

    let login_details = match login_service::validate_login(
        config,
        db_pool,
        auth_header.username,
        auth_header.password,
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to validate login for user due to err: {e}");
            return AccessTokenResponse::Error(
                String::from("Failed to authenticate user"),
                Status::Unauthorized,
            );
        }
    };

    let access_token = match oauth2_authorization_service::get_access_token_basic_auth(
        redis_pool,
        service,
        login_details.account_id,
    )
    .await
    {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to generate access token due to error {e:?}");
            return AccessTokenResponse::Error(
                String::from("Failed to generate access token"),
                Status::InternalServerError,
            );
        }
    };

    let response = access_token.into();
    AccessTokenResponse::Success(Json(response))
}
