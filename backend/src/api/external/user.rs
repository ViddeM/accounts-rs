use rocket::http::Status;
use rocket::State;
use serde::Serialize;
use sqlx::Pool;

use crate::api::auth::access_token_guard::AccessTokenAuth;
use crate::api::response::{ErrMsg, ResponseStatus};
use crate::db::DB;
use crate::services::user_info_service::{self, UserInfo, UserInfoError};

#[derive(Serialize, Clone)]
pub struct UserInfoResponse {
    first_name: String,
    last_name: String,
    email: String,
}

impl From<UserInfo> for UserInfoResponse {
    fn from(value: UserInfo) -> Self {
        UserInfoResponse {
            first_name: value.account.first_name,
            last_name: value.account.last_name,
            email: value.email,
        }
    }
}

#[get("/user")]
pub async fn get_user_info(
    access_token: AccessTokenAuth,
    db_pool: &State<Pool<DB>>,
) -> ResponseStatus<UserInfoResponse> {
    let user_info = match user_info_service::get_user_info(db_pool, access_token.access_token).await
    {
        Ok(user_info) => user_info,
        Err(UserInfoError::InvalidAccessToken) => {
            return ResponseStatus::err(Status::BadRequest, ErrMsg::InvalidAccessToken)
        }
        Err(err) => {
            error!("Failed to get user info {}", err);
            return ResponseStatus::internal_err();
        }
    };

    ResponseStatus::ok(user_info.into())
}
