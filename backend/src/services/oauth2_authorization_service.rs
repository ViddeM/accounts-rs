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
    #[error("There is no client with that client_id")]
    NoClientWithId,
    #[error("Redirect uri doesn't match client")]
    InvalidRedirectUri,
    #[error("Invalid client secret or redirect uri provided")]
    InvalidClient,
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

    let client = oauth_client_repository::get_by_client_id(&mut transaction, client_id.clone())
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
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    code: String,
) -> Result<(), Oauth2Error> {
    let mut transaction = new_transaction(db_pool).await?;

    let client = oauth_client_repository::get_by_client_id(&mut transaction, client_id)
        .await?
        .ok_or(Oauth2Error::NoClientWithId)?;

    if client.client_secret != client_secret || client.redirect_uri != redirect_uri {
        return Err(Oauth2Error::InvalidClient)?;
    }

    transaction.commit().await?;

    Ok(())
}
