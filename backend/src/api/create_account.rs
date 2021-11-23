use crate::db::account::AccountRepository;
use crate::util::config::Config;
use crate::util::response::{ErrMsg, ResponseStatus};
use rocket::http::Status;
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
    let existing_with_email = match account_repository.get_by_email(&create_account.email).await {
        Ok(val) => val,
        Err(err) => {
            error!("DB err: {:?}", err);
            return ResponseStatus::internal_err();
        }
    };

    if existing_with_email.is_none() {
        return ResponseStatus::err(Status::Ok, ErrMsg::EmailAlreadyRegistered);
    }

    ResponseStatus::ok(())
}
