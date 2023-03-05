use std::collections::HashMap;

use mobc_redis::RedisConnectionManager;
use rocket::{http::Status, State};
use serde::Serialize;
use sqlx::Pool;

use crate::{
    api::response::{ErrMsg, ResponseStatus},
    db::DB,
    services::oauth2_authorization_service::{self, Oauth2Error},
};

#[derive(Serialize, Clone)]
pub struct AccessTokenResponse {
    access_token: String,
    expires_in: u32,
    token_type: String,
}

const GRANT_TYPE_AUTHORIZATION_CODE: &str = "authorization_code";
const HEADER_CACHE_CONTROL: &str = "Cache-Control";
const HEADER_PRAGMA: &str = "Pragma";

const NO_CACHE: &str = "no-cache";
const NO_STORE: &str = "no-store";

const TOKEN_TYPE_BEARER: &str = "Bearer";

// Second step in the oauth2 authorization flow.
#[get("/token?<grant_type>&<redirect_uri>&<code>&<client_id>&<client_secret>")]
pub async fn get_access_token(
    db_pool: &State<Pool<DB>>,
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
    redirect_uri: String,
    code: String,
    client_id: String,
    client_secret: String,
    grant_type: String,
) -> ResponseStatus<AccessTokenResponse> {
    if grant_type != GRANT_TYPE_AUTHORIZATION_CODE {
        return ResponseStatus::err(Status::UnprocessableEntity, ErrMsg::InvalidGrantType);
    }

    match oauth2_authorization_service::get_access_token(
        db_pool,
        redis_pool,
        client_id,
        client_secret,
        redirect_uri,
        code,
    )
    .await
    {
        Ok(()) => {}
        Err(Oauth2Error::NoClientWithId) => {
            return ResponseStatus::err(Status::BadRequest, ErrMsg::InvalidClientId)
        }
        Err(Oauth2Error::InvalidRedirectUri) => {
            return ResponseStatus::err(Status::BadRequest, ErrMsg::InvalidRedirectUri)
        }
        Err(Oauth2Error::InvalidClientSecret) => {
            return ResponseStatus::err(Status::BadRequest, ErrMsg::InvalidClientSecret)
        }
        Err(Oauth2Error::InvalidCode) => {
            return ResponseStatus::err(Status::BadRequest, ErrMsg::InvalidCode)
        }
        Err(err) => {
            error!("Failed to get access token, err: {}", err);
            return ResponseStatus::internal_err();
        }
    }

    ResponseStatus::ok_with(
        AccessTokenResponse {
            access_token: String::new(), // TODO: Implement
            expires_in: 123,
            token_type: TOKEN_TYPE_BEARER.to_string(),
        },
        Status::Ok,
        HashMap::from([(HEADER_CACHE_CONTROL, NO_STORE), (HEADER_PRAGMA, NO_CACHE)]),
    )
}
