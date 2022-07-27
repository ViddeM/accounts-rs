use rocket::{serde::json::Json, State};
use serde::Serialize;

use crate::{
    api::response::ApiError,
    db::DB,
    services::{
        session_service::Session,
        user_service::{self, UserError},
    },
};

use crate::api::response::AccountsResponse;

#[derive(Serialize)]
pub struct MeResponse {
    first_name: String,
    last_name: String,
}

#[get("/me")]
pub async fn get_me(
    db_pool: &State<sqlx::Pool<DB>>,
    session: Session,
) -> Json<AccountsResponse<MeResponse>> {
    let acc = match user_service::get_me(session.account_id, db_pool).await {
        Ok(acc) => acc,
        Err(UserError::Internal) => {
            error!("An internal error occured whilst retrieving me");
            return Json(AccountsResponse::error(ApiError::InternalError));
        }
        Err(UserError::AccountNotFound) => {
            error!("Unable to find the account in the session!");
            // Here we should probably clear the session and require re-authorization, but for now return an error
            return Json(AccountsResponse::error(ApiError::InternalError));
        }
    };

    Json(AccountsResponse::success(MeResponse {
        first_name: acc.first_name,
        last_name: acc.last_name,
    }))
}
