use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};

use crate::{
    api::response::{NoContent, ResponseStatus},
    db::DB,
    services::{admin_session_service::AdminSession, whitelist_service},
};

#[derive(Serialize, Clone)]
pub struct WhitelistsResponse {
    emails: Vec<String>,
}

#[get("/whitelist")]
pub async fn get_whitelist(
    db_pool: &State<sqlx::Pool<DB>>,
    _admin_session: AdminSession,
) -> ResponseStatus<WhitelistsResponse> {
    let whitelist = match whitelist_service::get_whitelisted_emails(db_pool).await {
        Ok(whitelist) => whitelist,
        Err(err) => {
            error!("Failed to get whitelisted emails, err: {}", err);
            return ResponseStatus::internal_err();
        }
    };

    ResponseStatus::ok(WhitelistsResponse {
        emails: whitelist
            .into_iter()
            .map(|w| w.email)
            .collect::<Vec<String>>(),
    })
}

#[derive(Serialize, Deserialize)]
pub struct WhitelistRequest {
    email: String,
}

#[derive(Serialize, Clone)]
pub struct WhitelistResponse {
    email: String,
}

#[post("/whitelist", data = "<request>")]
pub async fn add_email_to_whitelist(
    db_pool: &State<sqlx::Pool<DB>>,
    _admin_session: AdminSession,
    request: Json<WhitelistRequest>,
) -> ResponseStatus<WhitelistResponse> {
    let whitelist = match whitelist_service::add_to_whitelist(db_pool, request.email.clone()).await
    {
        Ok(w) => w,
        Err(err) => {
            error!("Failed to add email to whitelist, err: {}", err);
            return ResponseStatus::internal_err();
        }
    };

    ResponseStatus::ok(WhitelistResponse {
        email: whitelist.email,
    })
}

#[delete("/whitelist/<email>")]
pub async fn delete_email_from_whitelist(
    db_pool: &State<sqlx::Pool<DB>>,
    _admin_session: AdminSession,
    email: String,
) -> ResponseStatus<NoContent> {
    match whitelist_service::delete_from_whitelist(db_pool, email).await {
        Ok(_) => ResponseStatus::<NoContent>::ok_no_content(),
        Err(err) => {
            error!("Failed to delete email from whitelist, err {}", err);
            ResponseStatus::internal_err()
        }
    }
}
