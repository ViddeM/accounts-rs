use chrono::{DateTime, Duration, Utc};
use mobc_redis::RedisConnectionManager;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rocket::State;
use serde::{Deserialize, Serialize};
use sqlx::Pool;
use uuid::Uuid;

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
    #[error("Failed to generate expiration time")]
    ExpirationTimeGeneration,
    #[error("Sqlx error")]
    SqlxError(#[from] sqlx::Error),
    #[error("Accounts error")]
    AccountsError(#[from] AccountsError),
    #[error("Redis error")]
    RedisError(#[from] RedisError),
    #[error("Failed to insert access token into the redis cache")]
    CacheInsertion,
}

const AUTH_TOKEN_LENGTH: usize = 48;

const AUTHORIZATION_KEY_REDIS_PREFIX: &str = "authorization_codes";
// 5 minutes
const AUTHORIZATION_CODE_EXPIRATION_SECONDS: usize = 5 * 60;

const ACCESS_TOKEN_LENGTH: usize = 128;
pub const ACCESS_TOKEN_KEY_REDIS_PREFIX: &str = "access_tokens";
// 1 hour
const ACCESS_TOKEN_EXPIRATION_SECONDS: i64 = 60 * 60;

#[derive(Deserialize, Serialize, Debug)]
struct AuthToken {
    code: String,
    client_id: String,
    account_id: Uuid,
}

pub async fn get_auth_token(
    db_pool: &State<Pool<DB>>,
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
    client_id: String,
    redirect_uri: String,
    state: String,
    account_id: Uuid,
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

    let auth_token = AuthToken {
        code: code.clone(),
        client_id,
        account_id,
    };

    let key = format!("{}:{}", AUTHORIZATION_KEY_REDIS_PREFIX, code);
    redis_service::redis_set(
        redis_pool,
        key,
        auth_token,
        AUTHORIZATION_CODE_EXPIRATION_SECONDS,
    )
    .await?;

    transaction.commit().await?;

    Ok(format!(
        "{}?state={}&code={}",
        client.redirect_uri, state, code
    ))
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AccessToken {
    pub access_token: String,
    pub expiration: DateTime<Utc>,
    pub client_id: String,
    pub account_id: Uuid,
}

pub async fn get_access_token(
    db_pool: &State<Pool<DB>>,
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    code: String,
) -> Result<AccessToken, Oauth2Error> {
    let mut transaction = new_transaction(db_pool).await?;

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

    let key = format!("{}:{}", AUTHORIZATION_KEY_REDIS_PREFIX, code);
    let code_auth_token: AuthToken = redis_service::redis_get_option(redis_pool, key.clone())
        .await?
        .ok_or(Oauth2Error::InvalidCode)?;

    if code_auth_token.client_id != client_id {
        error!(
            "Stored clientId for code(code={}) {} did not match provided client id {}",
            code, code_auth_token.client_id, client_id
        );
        return Err(Oauth2Error::NoClientWithId);
    }

    // The request has met the requirements, we can now issue the access token.
    // Start by deleting it from the cache
    redis_service::redis_del(redis_pool, key).await?;

    let access_token = generate_access_token(
        redis_pool,
        code_auth_token.client_id,
        code_auth_token.account_id,
    )
    .await?;

    transaction.commit().await?;

    Ok(access_token)
}

pub async fn get_access_token_basic_auth(
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
    service: String,
    account_id: Uuid,
) -> Result<AccessToken, Oauth2Error> {
    generate_access_token(redis_pool, service, account_id).await
}

async fn generate_access_token(
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
    client_id: String,
    account_id: Uuid,
) -> Result<AccessToken, Oauth2Error> {
    let access_token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(ACCESS_TOKEN_LENGTH)
        .map(char::from)
        .collect();

    let time_until_expiration = Duration::seconds(ACCESS_TOKEN_EXPIRATION_SECONDS);
    let expiration_time: DateTime<Utc> = Utc::now()
        .checked_add_signed(time_until_expiration)
        .ok_or(Oauth2Error::ExpirationTimeGeneration)?;

    let access_token: AccessToken = AccessToken {
        access_token: access_token.clone(),
        expiration: expiration_time,
        client_id,
        account_id,
    };

    let key = format!(
        "{}:{}",
        ACCESS_TOKEN_KEY_REDIS_PREFIX, access_token.access_token
    );
    redis_service::redis_set(
        redis_pool,
        key,
        access_token.clone(),
        time_until_expiration.num_seconds() as usize,
    )
    .await
    .or(Err(Oauth2Error::CacheInsertion))?;

    Ok(access_token)
}
