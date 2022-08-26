use mobc_redis::RedisConnectionManager;
use rocket::{http::CookieJar, response::Redirect, State};

use crate::services::{
    redis_service,
    session_service::{delete_session_cookie, Session},
};

#[post("/logout")]
pub async fn post_logout(
    session: Option<Session>,
    cookies: &CookieJar<'_>,
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
) -> Redirect {
    let session = match session {
        None => return Redirect::to("/"),
        Some(s) => s,
    };

    if let Err(e) = redis_service::redis_del(redis_pool, session.id).await {
        // This is bad but continue and at least try to remove the cookie from the user
        // We need to watch out for this error to avoid a memory-leak
        // (although one that can be solved by clearing the redis-db)
        error!("Failed to delete session from redis DB, err: {}", e);
    }

    delete_session_cookie(cookies).await;

    Redirect::to("/")
}
