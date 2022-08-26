use chrono::{DateTime, Duration, Utc};
use mobc::Pool;
use mobc_redis::redis::AsyncCommands;
use mobc_redis::RedisConnectionManager;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Request};
use rocket::State;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

use crate::models::login_details::LoginDetails;

use super::redis_service;

const SESSION_COOKIE_KEY: &str = "accounts_rs_session";
const SESSION_ID_LENGTH: usize = 48;
const SESSION_COOKIE_EXPIRATION_DAYS: i64 = 5;

const SESSIONS_KEY_PREFIX: &str = "sessions";
const ACCOUNT_SESSIONS_KEY_PREFIX: &str = "account_sessions";

#[derive(Debug)]
pub struct Session {
    pub id: String,
    pub expiration: DateTime<Utc>,
    pub account_id: Uuid,
}

#[derive(Deserialize, Serialize, Debug)]
struct RedisSession {
    pub id: String,
    pub expiration: DateTime<Utc>,
    pub account_id: uuid::Uuid,
}

impl From<RedisSession> for Session {
    fn from(s: RedisSession) -> Self {
        Self {
            id: s.id,
            expiration: s.expiration,
            account_id: Uuid::from_u128(s.account_id.as_u128()),
        }
    }
}

impl From<Session> for RedisSession {
    fn from(s: Session) -> Self {
        Self {
            id: s.id,
            expiration: s.expiration,
            account_id: uuid::Uuid::from_u128(s.account_id.as_u128()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("The client lacked a session cookie")]
    MissingCookie,
    #[error("Failed to retrieve redis pool")]
    RedisPoolError,
    #[error("Failed to generate session expiration time")]
    ExpirationTimeGeneration,
    #[error("Failed to insert session into the redis cache")]
    CacheInsertion,
    #[error("Failed to delete session from cache")]
    SessionDeletion,
    #[error("Failed to read value from redis cache")]
    CacheReadError,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = SessionError;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let session_cookie = match request.cookies().get_private(SESSION_COOKIE_KEY) {
            Some(s) => s,
            None => {
                return rocket::request::Outcome::Failure((
                    Status::Unauthorized,
                    SessionError::MissingCookie,
                ));
            }
        };
        let session_id = String::from(session_cookie.value());

        let redis_pool = match request
            .guard::<&State<mobc::Pool<RedisConnectionManager>>>()
            .await
            .succeeded()
        {
            Some(p) => p,
            None => {
                error!("Failed to retrieve redis pool");
                return rocket::request::Outcome::Failure((
                    Status::InternalServerError,
                    SessionError::RedisPoolError,
                ));
            }
        };

        let key = format!("{}:{}", SESSIONS_KEY_PREFIX, session_id);
        let session: Session =
            match redis_service::redis_get::<Option<RedisSession>>(redis_pool, key).await {
                Ok(Some(s)) => s.into(),
                Ok(None) => {
                    error!("Session was not found in the cache, this should generally not happen");
                    // Delete the invalid cookie and require re-login
                    delete_session_cookie(request.cookies()).await;
                    return rocket::request::Outcome::Failure((
                        Status::Unauthorized,
                        SessionError::MissingCookie,
                    ));
                }
                Err(err) => {
                    error!("Failed to retrieve session from redis, err: {}", err);
                    return rocket::request::Outcome::Failure((
                        Status::InternalServerError,
                        SessionError::RedisPoolError,
                    ));
                }
            };

        let now = Utc::now();
        if session.expiration < now {
            let key = format!("{}:{}", SESSIONS_KEY_PREFIX, session_id);

            // Session has expired, remove it from redis and cookie
            if let Err(e) = redis_service::redis_del(redis_pool, key).await {
                error!(
                    "Failed to delete expired session (id = {}) from redis, err: {}",
                    session_id, e
                );
                return rocket::request::Outcome::Failure((
                    Status::InternalServerError,
                    SessionError::RedisPoolError,
                ));
            }

            delete_session_cookie(request.cookies()).await;

            return rocket::request::Outcome::Failure((
                Status::Unauthorized,
                SessionError::MissingCookie,
            ));
        }

        rocket::request::Outcome::Success(session)
    }
}

pub async fn set_session(
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
    login_details: &LoginDetails,
    cookies: &CookieJar<'_>,
) -> Result<(), SessionError> {
    let session_id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(SESSION_ID_LENGTH)
        .map(char::from)
        .collect();

    let time_until_expiration = Duration::days(SESSION_COOKIE_EXPIRATION_DAYS);
    let expiration_time: DateTime<Utc> = Utc::now()
        .checked_add_signed(time_until_expiration)
        .ok_or(SessionError::ExpirationTimeGeneration)?;

    let session: RedisSession = Session {
        id: session_id.clone(),
        expiration: expiration_time,
        account_id: login_details.account_id,
    }
    .into();

    let key = format!("{}:{}", SESSIONS_KEY_PREFIX, session_id.clone());
    redis_service::redis_set(
        redis_pool,
        key,
        session,
        time_until_expiration.num_seconds() as usize,
    )
    .await
    .or(Err(SessionError::CacheInsertion))?;

    let key = format!(
        "{}:{}",
        ACCOUNT_SESSIONS_KEY_PREFIX, login_details.account_id
    );
    redis_service::redis_push(redis_pool, key, session_id.clone())
        .await
        .or(Err(SessionError::CacheInsertion))?;

    cookies.add_private(
        Cookie::build(SESSION_COOKIE_KEY, session_id)
            .secure(true)
            .finish(),
    );

    Ok(())
}

pub async fn delete_session_cookie<'r>(cookie_jar: &CookieJar<'r>) {
    if let Some(cookie) = cookie_jar.get_private(SESSION_COOKIE_KEY) {
        cookie_jar.remove_private(cookie);
    }
    // If the result is none then the cookie doesn't exist and we are all good
}

pub async fn reset_account_sessions(
    redis_pool: &State<Pool<RedisConnectionManager>>,
    account_id: Uuid,
) -> Result<(), SessionError> {
    let mut redis_conn = redis_pool.get().await.or_else(|err| {
        error!("Failed to get redis connection from pool, err {}", err);
        Err(SessionError::RedisPoolError)
    })?;

    let key = format!("{}:{}", ACCOUNT_SESSIONS_KEY_PREFIX, account_id);
    let list = redis_conn
        .lrange::<String, Vec<String>>(key.clone(), 0, -1)
        .await
        .or_else(|err| {
            error!("Failed to get list of sessions for account, err: {}", err);
            Err(SessionError::CacheReadError)
        })?;

    redis_conn
        .del::<Vec<String>, usize>(list)
        .await
        .or_else(|err| {
            error!("Failed to delete sessions for account, err: {}", err);
            Err(SessionError::SessionDeletion)
        })?;

    redis_conn.del::<String, ()>(key).await.or_else(|err| {
        error!(
            "Failed to delete list of sessions for account, err: {}",
            err
        );
        Err(SessionError::SessionDeletion)
    })?;

    Ok(())
}
