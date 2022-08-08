use rocket::{serde::json::Json, State};
use serde::Serialize;

use crate::{
    api::response::ApiError,
    db::DB,
    models::authority::AuthorityLevel,
    services::{admin_session_service::AdminSession, users_service},
};

use super::response::AccountsResponse;

#[derive(Serialize)]
pub struct Users {
    users: Vec<User>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    first_name: String,
    last_name: String,
    authority: AuthorityLevel,
}

#[get("/users")]
pub async fn get_users(
    db_pool: &State<sqlx::Pool<DB>>,
    _admin_session: AdminSession,
) -> Json<AccountsResponse<Users>> {
    let accs = match users_service::get_all_users(db_pool).await {
        Ok(accs) => accs,
        Err(err) => {
            error!("Failed to get all users, err: {}", err);
            return Json(AccountsResponse::error(ApiError::InternalError));
        }
    };
    Json(AccountsResponse::success(Users {
        users: accs
            .into_iter()
            .map(|acc| User {
                first_name: acc.first_name,
                last_name: acc.last_name,
                authority: acc.authority,
            })
            .collect::<Vec<User>>(),
    }))
}
