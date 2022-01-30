use crate::db::account::AccountRepository;
use crate::util::config::Config;
use crate::util::response::{ErrMsg, ResponseStatus};
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::content::Html;
use rocket::serde::json::Json;
use rocket::State;
use rocket_dyn_templates::Template;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[get("/create_account")]
pub async fn get_create_account() -> Html<Template> {
    let mut data: BTreeMap<&str, &str> = BTreeMap::new();
    Html(Template::render("create-account", &data))
}

#[derive(FromForm)]
pub struct CreateAccountForm {
    first_name: String,
    last_name: String,
    email: String,
    password: String,
    password_repeat: String,
}

#[post("/create_account", data = "<create_account>")]
pub async fn create_account(
    config: &State<Config>,
    account_repository: &State<AccountRepository>,
    create_account: Form<CreateAccountForm>,
) -> ResponseStatus<()> {
    let existing_with_email = match account_repository.get_by_email(&create_account.email).await {
        Ok(val) => val,
        Err(err) => {
            error!("DB err: {:?}", err);
            return ResponseStatus::internal_err();
        }
    };

    if !existing_with_email.is_none() {
        return ResponseStatus::err(Status::Ok, ErrMsg::EmailAlreadyRegistered);
    }

    ResponseStatus::ok(())
}
