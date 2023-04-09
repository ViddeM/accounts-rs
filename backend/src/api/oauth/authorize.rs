use mobc_redis::RedisConnectionManager;
use rocket::{http::Status, response::Redirect, State};
use sqlx::Pool;

use crate::{
    api::{
        auth::session_guard::Session,
        response::{ErrMsg, ResponseStatus},
    },
    db::DB,
    services::oauth2_authorization_service::{self, Oauth2Error},
};

const RESPONSE_TYPE_CODE: &str = "code";

/// First step in the oauth2 authorization flow.
#[get("/authorize?<response_type>&<client_id>&<redirect_uri>&<state>")]
pub async fn get_authorization(
    db_pool: &State<Pool<DB>>,
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
    response_type: String,
    client_id: String,
    redirect_uri: String,
    state: String,
    session: Session,
) -> Result<Redirect, ResponseStatus<()>> {
    if response_type != RESPONSE_TYPE_CODE {
        return Err(ResponseStatus::err(
            Status::UnprocessableEntity,
            ErrMsg::InvalidResponseType,
        ));
    }

    let url = match oauth2_authorization_service::get_auth_token(
        db_pool,
        redis_pool,
        client_id.clone(),
        redirect_uri,
        state,
        session.account_id,
    )
    .await
    {
        Ok(url) => url,
        Err(Oauth2Error::NoClientWithId) => {
            error!("No client with id '{}'", client_id);
            return Err(ResponseStatus::err(
                Status::BadRequest,
                ErrMsg::InvalidClientId,
            ));
        }
        Err(Oauth2Error::InvalidRedirectUri) => {
            return Err(ResponseStatus::err(
                Status::BadRequest,
                ErrMsg::InvalidRedirectUri,
            ))
        }
        Err(err) => {
            panic!("An oauth2 error occurred, err: {err}");
        }
    };

    Ok(Redirect::found(url))
}
