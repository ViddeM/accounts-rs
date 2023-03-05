use mobc_redis::RedisConnectionManager;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rocket::State;
use sqlx::Pool;

use crate::{
    db::{new_transaction, oauth_client_repository, DB},
    util::accounts_error::AccountsError,
};

use super::redis_service::{self, RedisError};

#[derive(Debug, thiserror::Error)]
pub enum Oauth2Error {
    #[error("Client ID did not match the provided code")]
    InvalidClientId,
    #[error("There is no client with that client_id")]
    NoClientWithId,
    #[error("Redirect uri doesn't match client")]
    InvalidRedirectUri,
    #[error("Invalid client secret")]
    InvalidClientSecret,
    #[error("Invalid authorization code provided")]
    InvalidCode,
    #[error("Sqlx error")]
    SqlxError(#[from] sqlx::Error),
    #[error("Accounts error")]
    AccountsError(#[from] AccountsError),
    #[error("Redis error")]
    RedisError(#[from] RedisError),
}

const AUTH_TOKEN_LENGTH: usize = 48;
const AUTHORIZATION_KEY_REDIS_PREFIX: &str = "authorization_codes";
// 30 minutes
const AUTHORIZATION_CODE_EXPIRATION_SECONDS: usize = 30 * 60;

pub async fn get_auth_token(
    db_pool: &State<Pool<DB>>,
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
    client_id: String,
    redirect_uri: String,
    state: String,
) -> Result<String, Oauth2Error> {
    let mut transaction = new_transaction(db_pool).await?;

    let client = oauth_client_repository::get_by_client_id(&mut transaction, &client_id)
        .await?
        .ok_or(Oauth2Error::NoClientWithId)?;

    if redirect_uri != client.redirect_uri {
        error!(
            "Redirect uri doesn't match, request redirect_uri: {}, client set redirect_uri: {}",
            redirect_uri, client.redirect_uri
        );
        return Err(Oauth2Error::InvalidRedirectUri);
    }

    let code: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(AUTH_TOKEN_LENGTH)
        .map(char::from)
        .collect();

    let key = format!("{}:{}", AUTHORIZATION_KEY_REDIS_PREFIX, code);
    redis_service::redis_set(
        redis_pool,
        key,
        client_id,
        AUTHORIZATION_CODE_EXPIRATION_SECONDS,
    )
    .await?;

    transaction.commit().await?;

    Ok(format!(
        "{}?state={}&code={}",
        client.redirect_uri, state, code
    ))
}

pub async fn get_access_token(
    db_pool: &State<Pool<DB>>,
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    code: String,
) -> Result<(), Oauth2Error> {
    let mut transaction = new_transaction(db_pool).await?;

    let key = format!("{}:{}", AUTHORIZATION_KEY_REDIS_PREFIX, code);
    let code_client_id: String = redis_service::redis_get_option(redis_pool, key)
        .await?
        .ok_or(Oauth2Error::InvalidCode)?;

    if code_client_id != client_id {
        error!(
            "Stored clientId for code(code={}) {} did not match provided client id {}",
            code, code_client_id, client_id
        );
        return Err(Oauth2Error::NoClientWithId);
    }

    let client = oauth_client_repository::get_by_client_id(&mut transaction, &client_id)
        .await?
        .ok_or(Oauth2Error::NoClientWithId)?;

    if client.redirect_uri != redirect_uri {
        error!(
            "Received redirect_uri ({}) did not match stored redirect_uri ({}) for client {}",
            redirect_uri, client.redirect_uri, client_id
        );
        return Err(Oauth2Error::InvalidRedirectUri);
    }

    if client.client_secret != client_secret {
        error!(
            "Received client_secret did not match stored client_secret for client {}",
            client_id
        );
        return Err(Oauth2Error::InvalidClientSecret);
    }

    transaction.commit().await?;

    Ok(())
}
