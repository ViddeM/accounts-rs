use crate::services::create_account_service;
use crate::util::config::Config;
use rocket::form::Form;

use rocket::State;
use rocket_dyn_templates::Template;

use crate::db::DB;
use crate::services::create_account_service::CreateAccountError;
use sqlx::Pool;
use std::collections::BTreeMap;

use super::password_validation::{get_default_password_page_data, validate_new_passwords};

const ERROR_KEY: &str = "error";
const INFO_KEY: &str = "info";
const FIRST_NAME_KEY: &str = "first_name";
const LAST_NAME_KEY: &str = "last_name";
const EMAIL_KEY: &str = "email";

const ERR_EMAIL_NOT_WHITELISTED: &str = "The provided email is not in the whitelist";
const ERR_INTERNAL: &str = "An internal error has occured, please try again later";

const INFO_EMAIL_HAS_BEEN_SENT: &str = "Your account has been created but has yet to be activated,\
                                        an email has been sent to the provided email address with \
                                        steps to activate your account.";

const CREATE_ACCOUNT_TEMPLATE_NAME: &str = "create-account";

fn create_account_error(data: &mut BTreeMap<&str, String>, error: &str) -> Template {
    data.insert(ERROR_KEY, error.to_string());
    Template::render(CREATE_ACCOUNT_TEMPLATE_NAME, &data)
}

#[get("/create_account")]
pub async fn get_create_account() -> Template {
    let data = get_default_password_page_data();
    Template::render(CREATE_ACCOUNT_TEMPLATE_NAME, data)
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
    db_pool: &State<Pool<DB>>,
    create_account: Form<CreateAccountForm>,
) -> Template {
    let mut data = get_default_password_page_data();

    data.insert(FIRST_NAME_KEY, create_account.first_name.to_string());
    data.insert(LAST_NAME_KEY, create_account.last_name.to_string());
    data.insert(EMAIL_KEY, create_account.email.to_string());

    if let Err(e) =
        validate_new_passwords(&create_account.password, &create_account.password_repeat)
    {
        return create_account_error(&mut data, e);
    }

    if let Err(e) = create_account_service::create_account(
        config,
        db_pool,
        create_account.first_name.to_owned(),
        create_account.last_name.to_owned(),
        create_account.email.to_owned(),
        create_account.password.to_owned(),
    )
    .await
    {
        match e {
            CreateAccountError::Internal | CreateAccountError::EmailError(_) => {
                return create_account_error(&mut data, ERR_INTERNAL)
            }
            CreateAccountError::EmailNotWhitelisted => {
                return create_account_error(&mut data, ERR_INTERNAL)
            }
            CreateAccountError::EmailInUse => {
                data.insert(INFO_KEY, INFO_EMAIL_HAS_BEEN_SENT.to_string());
                return Template::render(CREATE_ACCOUNT_TEMPLATE_NAME, &data);
            }
        }
    };

    data.insert(INFO_KEY, INFO_EMAIL_HAS_BEEN_SENT.to_string());
    return Template::render(CREATE_ACCOUNT_TEMPLATE_NAME, &data);
}
