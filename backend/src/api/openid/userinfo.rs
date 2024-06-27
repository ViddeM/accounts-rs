use rocket::{serde::json::Json, State};
use serde::Serialize;
use sqlx::Pool;

use crate::{
    api::auth::access_token_guard::AccessTokenAuth,
    db::DB,
    services::user_info_service::{self, UserInfoError},
};

#[derive(Debug, Clone, Serialize)]
pub struct Userinfo {
    email: String,
}

#[derive(Responder, Debug)]
pub enum UserinfoResponse {
    #[response(status = 200)]
    Success(Json<Userinfo>),
    #[response(status = 401)]
    Unauthorized(String),
    #[response(status = 500)]
    Failure(String),
}

#[get("/userinfo")]
pub async fn get_userinfo(
    access_token: AccessTokenAuth,
    db_pool: &State<Pool<DB>>,
) -> UserinfoResponse {
    let user_info = match user_info_service::get_user_info(db_pool, access_token.access_token).await
    {
        Ok(user_info) => user_info,
        Err(UserInfoError::InvalidAccessToken) => {
            return UserinfoResponse::Unauthorized("Invalid access token".to_string())
        }
        Err(err) => {
            error!("Failed to get user info {}", err);
            return UserinfoResponse::Failure("An unknown error occurred".to_string());
        }
    };

    UserinfoResponse::Success(Json(Userinfo {
        email: user_info.email,
    }))
}
