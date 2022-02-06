use crate::services::create_account_service;
use crate::util::config::Config;
use rocket::form::Form;
use rocket::response::content::Html;

use rocket::{Either, State};
use rocket_dyn_templates::Template;

use crate::db::DB;
use crate::services::create_account_service::CreateAccountError;
use rocket::response::Redirect;
use sqlx::Pool;
use std::collections::BTreeMap;

const ERROR_KEY: &str = "error";
const FIRST_NAME_KEY: &str = "first_name";
const LAST_NAME_KEY: &str = "last_name";
const EMAIL_KEY: &str = "email";
const MIN_PASSWORD_LEN_KEY: &str = "min_password_len";
const MAX_PASSWORD_LEN_KEY: &str = "max_password_len";

const ERR_PASSWORDS_NOT_MATCH: &str = "Passwords does not match";
const ERR_PASSWORD_TOO_SHORT: &str = "The password is too short";
const ERR_PASSWORD_TOO_LONG: &str = "The password is too long";
const ERR_EMAIL_IN_USE: &str = "That email is already in use";
const ERR_INTERNAL: &str = "An internal error has occured, please try again later";

const CREATE_ACCOUNT_TEMPLATE_NAME: &str = "create-account";

const LOGIN_PAGE_URL: &str = "/api/login";

const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 128;

fn get_default_create_account_data() -> BTreeMap<&'static str, String> {
    let mut data: BTreeMap<&str, String> = BTreeMap::new();
    let min_password_length = MIN_PASSWORD_LENGTH.to_string();
    let max_password_length = MAX_PASSWORD_LENGTH.to_string();
    data.insert(MIN_PASSWORD_LEN_KEY, min_password_length.to_string());
    data.insert(MAX_PASSWORD_LEN_KEY, max_password_length.to_string());
    data
}

fn create_account_error(data: &mut BTreeMap<&str, String>, error: &str) -> Html<Template> {
    data.insert(ERROR_KEY, error.to_string());
    Html(Template::render(CREATE_ACCOUNT_TEMPLATE_NAME, &data))
}

#[get("/create_account")]
pub async fn get_create_account() -> Html<Template> {
    let data = get_default_create_account_data();
    Html(Template::render(CREATE_ACCOUNT_TEMPLATE_NAME, data))
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
) -> Either<Html<Template>, Redirect> {
    let mut data = get_default_create_account_data();

    data.insert(FIRST_NAME_KEY, create_account.first_name.to_string());
    data.insert(LAST_NAME_KEY, create_account.last_name.to_string());
    data.insert(EMAIL_KEY, create_account.email.to_string());

    if create_account.password != create_account.password_repeat {
        return Either::Left(create_account_error(&mut data, ERR_PASSWORDS_NOT_MATCH));
    }

    if create_account.password.len() < MIN_PASSWORD_LENGTH {
        return Either::Left(create_account_error(&mut data, ERR_PASSWORD_TOO_SHORT));
    }

    if create_account.password.len() > MAX_PASSWORD_LENGTH {
        return Either::Left(create_account_error(&mut data, ERR_PASSWORD_TOO_LONG));
    }

    if let Err(e) = create_account_service::create_account(
        config,
        db_pool,
        &create_account.first_name,
        &create_account.last_name,
        &create_account.email,
        &create_account.password,
    )
    .await
    {
        return Either::Left(create_account_error(&mut data, e.to_api_err()));
    };

    Either::Right(Redirect::to(LOGIN_PAGE_URL))
}

impl CreateAccountError {
    fn to_api_err<'a>(self) -> &'a str {
        match self {
            CreateAccountError::Internal => ERR_INTERNAL,
            CreateAccountError::EmailInUse => ERR_EMAIL_IN_USE,
        }
    }
}
