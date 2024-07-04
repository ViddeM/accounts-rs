use std::str::from_utf8;

use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, Utc};
use jwt::{PKeyWithDigest, SignWithKey};
use mobc_redis::RedisConnectionManager;
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Private},
    rsa::Rsa,
};
use rocket::{
    form::Form,
    http::{Header, Status},
    request::{self, FromRequest},
    serde::json::Json,
    Request, State,
};
use serde::Serialize;
use sqlx::Pool;

use crate::{
    db::DB,
    models::{id_token::IdToken, oauth_scope::OauthScope},
    services::{
        login_service,
        oauth_authorization_service::{self, AccessToken, Oauth2Error},
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

#[derive(Debug, Clone)]
enum CacheControlVariant {
    NoStore,
}

#[derive(Debug, Clone)]
struct CacheControl(CacheControlVariant);

impl From<CacheControl> for Header<'static> {
    fn from(value: CacheControl) -> Self {
        Self {
            name: HEADER_CACHE_CONTROL.into(),
            value: match value.0 {
                CacheControlVariant::NoStore => NO_STORE.into(),
            },
        }
    }
}

#[derive(Debug, Clone)]
enum PragmaVariant {
    NoCache,
}

#[derive(Debug, Clone)]
struct Pragma(PragmaVariant);

impl From<Pragma> for Header<'static> {
    fn from(value: Pragma) -> Self {
        Self {
            name: HEADER_PRAGMA.into(),
            value: match value.0 {
                PragmaVariant::NoCache => NO_CACHE.into(),
            },
        }
    }
}

#[derive(Responder, Debug, Clone)]
pub enum AccessTokenResponse {
    #[response(status = 200)]
    Success {
        inner: Json<AccessTokenResponseData>,
        cache_control: CacheControl,
        pragma: Pragma,
    },
    #[response(status = 400)]
    Error(Json<AccessTokenErrorResponse>),
    #[response(status = 500)]
    InternalError(String),
}

#[derive(Serialize, Clone, Debug)]
pub struct AccessTokenResponseData {
    access_token: String,
    expires_in: u32,
    token_type: String,
    id_token: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct AccessTokenErrorResponse {
    error: AccessTokenError,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AccessTokenError {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnsupportedGrantType,
    InvalidScope,
}

// Second step in the oauth2 authorization flow.
#[post("/token", data = "<request>")]
pub async fn post_access_token(
    db_pool: &State<Pool<DB>>,
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
    config: &State<Config>,
    request: Form<AccessTokenRequest>,
) -> AccessTokenResponse {
    if request.grant_type != GRANT_TYPE_AUTHORIZATION_CODE {
        log::warn!("Unsupported grant type {}", request.grant_type);
        return AccessTokenResponse::Error(Json(AccessTokenErrorResponse {
            error: AccessTokenError::UnsupportedGrantType,
        }));
    }

    let access_token = match oauth_authorization_service::get_access_token(
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
            log::warn!("Received invalid client ID: {}", request.client_id);
            return AccessTokenResponse::Error(Json(AccessTokenErrorResponse {
                error: AccessTokenError::InvalidClient,
            }));
        }
        Err(Oauth2Error::InvalidRedirectUri) => {
            log::warn!("Received invalid client ID: {}", request.client_id);
            return AccessTokenResponse::Error(Json(AccessTokenErrorResponse {
                error: AccessTokenError::InvalidRequest,
            }));
        }
        Err(Oauth2Error::InvalidClientSecret) => {
            log::warn!("Received invalid client secret");
            return AccessTokenResponse::Error(Json(AccessTokenErrorResponse {
                error: AccessTokenError::InvalidRequest,
            }));
        }
        Err(Oauth2Error::InvalidCode) => {
            log::warn!("Received an invalid auth code: {}", request.code);
            return AccessTokenResponse::Error(Json(AccessTokenErrorResponse {
                error: AccessTokenError::InvalidGrant,
            }));
        }
        Err(err) => {
            error!("Failed to get access token, err: {}", err);
            return AccessTokenResponse::InternalError("Internal error".into());
        }
    };

    let id_token = if access_token.has_scope(&OauthScope::OpenId) {
        match get_id_token(&config, &access_token) {
            Ok(t) => Some(t),
            Err(err) => {
                log::error!("Failed to create or sign ID token JWT, err: {err}");
                return AccessTokenResponse::InternalError("Internal error".into());
            }
        }
    } else {
        None
    };

    let access_token_response = AccessTokenResponseData {
        access_token: access_token.access_token.clone(),
        expires_in: access_token.expires_in(),
        token_type: TOKEN_TYPE_BEARER.into(),
        id_token,
    };

    AccessTokenResponse::Success {
        inner: Json(access_token_response),
        cache_control: CacheControl(CacheControlVariant::NoStore),
        pragma: Pragma(PragmaVariant::NoCache),
    }
}

fn calculate_expire_time(expiration: DateTime<Utc>) -> u32 {
    let now = Utc::now();
    let expires_in = expiration.timestamp() - now.timestamp(); // The number of seconds until expiration
    if expires_in <= 0 {
        log::warn!("Expires in is {expires_in} before being returned to the caller!");
    }
    expires_in as u32
}

fn get_id_token(config: &Config, access_token: &AccessToken) -> Result<String, String> {
    let id_token = IdToken {
        issuer: config.backend_address.clone(),
        subject: access_token.account_id.into(),
        audience: access_token.client_id.clone(),
        expires_at: access_token.expiration.timestamp(),
        issued_at: access_token.issued_at.timestamp(),
    };

    let jwt_header = jwt::Header {
        algorithm: jwt::AlgorithmType::Rs256,
        key_id: None,
        type_: Some(jwt::header::HeaderType::JsonWebToken),
        content_type: None,
    };

    let signing_key: Rsa<Private> = config.jwt_signing_key.clone();
    let key = PKeyWithDigest {
        digest: MessageDigest::sha256(),
        key: PKey::from_rsa(signing_key)
            .map_err(|err| format!("Failed to create signing key from rsa key, err: {err:?}"))?,
    };

    let signed_token = jwt::Token::new(jwt_header, id_token)
        .sign_with_key(&key)
        .map_err(|err| format!("Failed to sign jwt token, err: {err:?}"))?;

    Ok(signed_token.into())
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
    // TODO: Investigate what/how/if these should be used.
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

    let access_token = match oauth_authorization_service::get_access_token_basic_auth(
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
