use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};

use crate::{
    api::response::ApiError,
    db::DB,
    models::whitelist::Whitelist,
    services::{admin_session_service::AdminSession, whitelist_service},
};

use super::response::AccountsResponse;

#[derive(Serialize)]
pub struct WhitelistsResponse {
    emails: Vec<String>,
}

#[get("/whitelist")]
pub async fn get_whitelist(
    db_pool: &State<sqlx::Pool<DB>>,
    _admin_session: AdminSession,
) -> Json<AccountsResponse<WhitelistsResponse>> {
    let whitelist = match whitelist_service::get_whitelisted_emails(db_pool).await {
        Ok(whitelist) => whitelist,
        Err(err) => {
            error!("Failed to get whitelisted emails, err: {}", err);
            return Json(AccountsResponse::error(ApiError::InternalError));
        }
    };

    Json(AccountsResponse::success(WhitelistsResponse {
        emails: whitelist
            .into_iter()
            .map(|w| w.email)
            .collect::<Vec<String>>(),
    }))
}

#[derive(Serialize, Deserialize)]
pub struct WhitelistRequest {
    email: String,
}

#[derive(Serialize)]
pub struct WhitelistResponse {
    email: String,
}

#[post("/whitelist", data = "<request>")]
pub async fn add_email_to_whitelist(
    db_pool: &State<sqlx::Pool<DB>>,
    _admin_session: AdminSession,
    request: Json<WhitelistRequest>,
) -> Json<AccountsResponse<WhitelistResponse>> {
    let whitelist = match whitelist_service::add_to_whitelist(db_pool, request.email.clone()).await
    {
        Ok(w) => w,
        Err(err) => {
            error!("Failed to add email to whitelist, err: {}", err);
            return Json(AccountsResponse::error(ApiError::InternalError));
        }
    };

    Json(AccountsResponse::success(WhitelistResponse {
        email: whitelist.email,
    }))
}

#[delete("/whitelist/<email>")]
pub async fn delete_email_from_whitelist(
    db_pool: &State<sqlx::Pool<DB>>,
    _admin_session: AdminSession,
    email: String,
) -> Json<AccountsResponse<()>> {
    return Json(
        match whitelist_service::delete_from_whitelist(db_pool, email).await {
            Ok(_) => AccountsResponse::success(()),
            Err(err) => {
                error!("Failed to delete email from whitelist, err {}", err);
                AccountsResponse::error(ApiError::InternalError)
            }
        },
    );
}
