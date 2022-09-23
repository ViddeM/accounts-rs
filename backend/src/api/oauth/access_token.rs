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
}

const GRANT_TYPE_AUTHORIZATION_CODE: &str = "authorization_code";

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
        Err(err) => {
            error!("Failed to get access token, err: {}", err);
            return ResponseStatus::internal_err();
        }
    }

    ResponseStatus::ok(AccessTokenResponse {
        access_token: String::new(),
        expires_in: 123,
    })
}
