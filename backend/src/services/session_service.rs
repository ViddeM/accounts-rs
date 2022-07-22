use chrono::{DateTime, Duration, Utc};
use mobc_redis::redis::{AsyncCommands, FromRedisValue, ToRedisArgs};
use mobc_redis::RedisConnectionManager;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Request};
use rocket::State;
use sqlx::types::uuid::Uuid;

use crate::models::login_details::LoginDetails;

const SESSION_COOKIE_KEY: &str = "accounts_rs_session";
const SESSION_ID_LENGTH: usize = 48;
const SESSION_COOKIE_EXPIRATION_DAYS: i64 = 5;

#[derive(Debug)]
pub struct Session {
    pub id: String,
    pub expiration: DateTime<Utc>,
    pub user_id: Uuid,
}

impl ToRedisArgs for Session {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + mobc_redis::redis::RedisWrite,
    {
        let data = format!(
            "ID={}::EXPIRATION={}::USER_ID={}",
            self.id,
            self.expiration.to_string(),
            self.user_id
        );

        out.write_arg(data.as_bytes());
    }
}

macro_rules! invalid_type_error {
    ($v:expr, $det:expr) => {{
        mobc_redis::redis::RedisError::from((
            mobc_redis::redis::ErrorKind::TypeError,
            "Response was of incompatible type",
            format!("{:?} (response was {:?})", $det, $v),
        ))
    }};
}

impl FromRedisValue for Session {
    fn from_redis_value(v: &mobc_redis::redis::Value) -> mobc_redis::redis::RedisResult<Self> {
        let vals = String::from_redis_value(v)?
            .as_str()
            .split("::")
            .map(|v| {
                let (a, b) = v.split_once("=").ok_or(invalid_type_error!("Tuple", v))?;
                Ok((String::from(a), String::from(b)))
            })
            .collect::<mobc_redis::redis::RedisResult<Vec<(String, String)>>>()?;

        let mut id: Option<String> = None;
        let mut expiration: Option<DateTime<Utc>> = None;
        let mut user_id: Option<Uuid> = None;
        for (k, v) in vals.into_iter() {
            match k.as_str() {
                "ID" => {
                    id = Some(String::from(v));
                }
                "EXPIRATION" => {
                    let expiration_date: DateTime<Utc> = v
                        .parse()
                        .ok()
                        .ok_or(invalid_type_error!("DateTime<Utc>", v))?;
                    expiration = Some(expiration_date);
                }
                "USER_ID" => {
                    user_id = Some(
                        Uuid::parse_str(&v)
                            .ok()
                            .ok_or(invalid_type_error!("Uuid", v))?,
                    )
                }
                _ => { /* Ignore unexpected keys */ }
            }
        }

        Ok(Session {
            id: id.expect("Failed to retrieve ID for session from cache"),
            expiration: expiration.expect("Failed to retrieve expiration for session from cache"),
            user_id: user_id.expect("Failed to retrieve user_id for session from cache"),
        })
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

        let mut redis_conn = match redis_pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get connection to redis DB, err: {}", e);
                return rocket::request::Outcome::Failure((
                    Status::InternalServerError,
                    SessionError::RedisPoolError,
                ));
            }
        };

        let session = match redis_conn
            .get::<String, Option<Session>>(session_id.clone())
            .await
        {
            Ok(Some(s)) => s,
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
            // Session has expired, remove it from redis and cookie
            if let Err(e) = delete_session_from_cache(&mut redis_conn, session_id.clone()).await {
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
    let mut redis_conn = redis_pool.get().await.or_else(|err| {
        error!("Failed to retrieve redis pool, err: {}", err);
        Err(SessionError::RedisPoolError)
    })?;

    let session_id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(SESSION_ID_LENGTH)
        .map(char::from)
        .collect();

    let expiration_time: DateTime<Utc> = Utc::now()
        .checked_add_signed(Duration::days(SESSION_COOKIE_EXPIRATION_DAYS))
        .ok_or(SessionError::ExpirationTimeGeneration)?;

    let session = Session {
        id: session_id.clone(),
        expiration: expiration_time,
        user_id: login_details.account_id,
    };

    redis_conn
        .set::<String, Session, String>(session_id.clone(), session)
        .await
        .or_else(|err| {
            error!("Failed to insert session in the redis cache, err: {}", err);
            Err(SessionError::CacheInsertion)
        })?;

    cookies.add_private(
        Cookie::build(SESSION_COOKIE_KEY, session_id)
            .secure(true)
            .finish(),
    );

    Ok(())
}

pub async fn delete_session_from_cache(
    redis_conn: &mut mobc::Connection<RedisConnectionManager>,
    session_id: String,
) -> Result<(), SessionError> {
    redis_conn
        .del::<String, ()>(session_id)
        .await
        .or(Err(SessionError::SessionDeletion))
}

pub async fn delete_session_cookie<'r>(cookie_jar: &CookieJar<'r>) {
    if let Some(cookie) = cookie_jar.get_private(SESSION_COOKIE_KEY) {
        cookie_jar.remove_private(cookie);
    }
    // If the result is none then the cookie doesn't exist and we are all good
}
