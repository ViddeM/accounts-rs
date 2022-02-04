use crate::db::login_details::LoginDetailsRepository;
use crate::util::config::Config;
use crate::util::response::{ErrMsg, ResponseStatus};
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::content::Html;

use rocket::{Either, State};
use rocket_dyn_templates::Template;

use rocket::response::Redirect;
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
const ERR_UNKNOWN: &str = "An unknown error has occured, please try again later";

const CREATE_ACCOUNT_TEMPLATE_NAME: &str = "create-account";

const LOGIN_PAGE_URL: &str = "/api/login";

const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 128;

fn get_default_create_account_data() -> BTreeMap<&'static str, &'static str> {
    let mut data: BTreeMap<&str, &str> = BTreeMap::new();
    let min_password_length = MIN_PASSWORD_LENGTH.to_string();
    let max_password_length = MAX_PASSWORD_LENGTH.to_string();
    data.insert(MIN_PASSWORD_LEN_KEY, min_password_length.as_str().clone());
    data.insert(MAX_PASSWORD_LEN_KEY, max_password_length.as_str().clone());
    data
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
    _config: &State<Config>,
    login_details_repository: &State<LoginDetailsRepository>,
    create_account: Form<CreateAccountForm>,
) -> Either<Html<Template>, Redirect> {
    let mut data = get_default_create_account_data();

    data.insert(FIRST_NAME_KEY, &create_account.first_name);
    data.insert(LAST_NAME_KEY, &create_account.last_name);
    data.insert(EMAIL_KEY, &create_account.email);

    if create_account.password != create_account.password_repeat {
        data.insert(ERROR_KEY, ERR_PASSWORDS_NOT_MATCH);
        return Either::Left(Html(Template::render(CREATE_ACCOUNT_TEMPLATE_NAME, &data)));
    }

    if create_account.password.len() < MIN_PASSWORD_LENGTH {
        data.insert(ERROR_KEY, ERR_PASSWORD_TOO_SHORT);
        return Either::Left(Html(Template::render(CREATE_ACCOUNT_TEMPLATE_NAME, &data)));
    }

    if create_account.password.len() > MAX_PASSWORD_LENGTH {
        data.insert(ERROR_KEY, ERR_PASSWORD_TOO_LONG);
        return Either::Left(Html(Template::render(CREATE_ACCOUNT_TEMPLATE_NAME, &data)));
    }

    let existing_with_email = match login_details_repository
        .get_by_email(&create_account.email)
        .await
    {
        Ok(val) => val,
        Err(err) => {
            error!("DB err: {:?}", err);
            data.insert(ERROR_KEY, ERR_UNKNOWN);
            return Either::Left(Html(Template::render("create-account", data)));
        }
    };

    if existing_with_email.is_some() {
        data.insert(ERROR_KEY, ERR_EMAIL_IN_USE);
        return Either::Left(Html(Template::render(CREATE_ACCOUNT_TEMPLATE_NAME, data)));
    }

    Either::Right(Redirect::to(LOGIN_PAGE_URL))
}
