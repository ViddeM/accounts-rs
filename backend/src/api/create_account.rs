use crate::config::Config;
use crate::db::account::AccountRepository;
use crate::response::ResponseStatus;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CreateAccountRequest {
    first_name: Option<String>,
    last_name: Option<String>,
    email: String,
    password: String,
}

#[post("/create_account", format = "json", data = "<create_account>")]
pub async fn create_account(
    config: &State<Config>,
    account_repository: &State<AccountRepository>,
    create_account: Json<CreateAccountRequest>,
) -> ResponseStatus<()> {
    let existing_with_email = account_repository
        .get_by_email(&create_account.email)
        .await?;
}
