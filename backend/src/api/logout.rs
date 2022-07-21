use mobc_redis::RedisConnectionManager;
use rocket::{http::CookieJar, response::Redirect, State};

use crate::services::session_service::{delete_session_cookie, delete_session_from_cache, Session};

#[post("/logout")]
pub async fn post_logout(
    session: Option<Session>,
    cookies: &CookieJar<'_>,
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
) -> Redirect {
    let session = match session {
        None => return Redirect::to("/api/login"),
        Some(s) => s,
    };

    match redis_pool.get().await {
        Ok(mut conn) => {
            if let Err(e) = delete_session_from_cache(&mut conn, session.id).await {
                // This is bad but continue and at least try to remove the cookie from the user
                // We need to watch out for this error to avoid a memory-leak
                // (although one that can be solved by clearing the redis-db)
                error!("Failed to delete session from redis DB, err: {}", e);
            }
        }
        Err(e) => {
            // Also here we can have a memory leak but let's continue and hopefully our users won't notice at least.
            error!("Failed to get connection to redis DB, err: {}", e);
        }
    };

    delete_session_cookie(cookies).await;

    Redirect::to("/api/login")
}
