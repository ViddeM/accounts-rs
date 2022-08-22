use rocket::State;
use serde::Serialize;

use crate::{
    api::response::ResponseStatus,
    db::DB,
    models::authority::AuthorityLevel,
    services::{
        session_service::Session,
        user_service::{self, UserError},
    },
};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    first_name: String,
    last_name: String,
    email: Option<String>,
    authority: AuthorityLevel,
}

#[get("/me")]
pub async fn get_me(
    db_pool: &State<sqlx::Pool<DB>>,
    session: Session,
) -> ResponseStatus<MeResponse> {
    let resp = match user_service::get_me(session.account_id, db_pool).await {
        Ok((acc, login_details)) => MeResponse {
            first_name: acc.first_name,
            last_name: acc.last_name,
            email: match login_details {
                Some(l) => Some(l.email),
                None => None,
            },
            authority: acc.authority,
        },
        Err(UserError::Internal) => {
            error!("An internal error occured whilst retrieving me");
            return ResponseStatus::internal_err();
        }
        Err(UserError::AccountNotFound) => {
            error!("Unable to find the account in the session!");
            // Here we should probably clear the session and require re-authorization, but for now return an error
            return ResponseStatus::internal_err();
        }
    };

    ResponseStatus::ok(resp)
}
