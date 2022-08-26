use rocket::{http::Status, State};
use serde::Serialize;
use sqlx::Pool;

use crate::{
    api::response::{ErrMsg, ResponseStatus},
    db::DB,
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
    redirect_uri: String,
    code: String,
    client_id: String,
    client_secret: String,
    grant_type: String,
) -> ResponseStatus<AccessTokenResponse> {
    if grant_type != GRANT_TYPE_AUTHORIZATION_CODE {
        return ResponseStatus::err(Status::UnprocessableEntity, ErrMsg::InvalidGrantType);
    }

    ResponseStatus::ok(AccessTokenResponse {
        access_token: String::new(),
        expires_in: 123,
    })
}
