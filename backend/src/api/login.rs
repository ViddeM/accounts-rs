use crate::db::login_details_repository;
use crate::db::{new_transaction, DB};
use crate::services::password_service;
use crate::util::config::Config;
use rocket::form::Form;
use rocket::response::content::Html;
use rocket::response::Redirect;
use rocket::{Either, State};
use rocket_dyn_templates::Template;
use sqlx::Pool;
use std::collections::BTreeMap;

const LOGIN_TEMPLATE_NAME: &str = "login";

const ERROR_KEY: &str = "error";

const ERR_INVALID_EMAIL_OR_PASSWORD: &str = "Invalid email or password";
const ERR_ACCOUNT_NOT_ACTIVATED: &str = "The account has not yet been activated";
const ERR_INTERNAL: &str = "An internal error occurred";

const MIN_PASSWORD_LEN_KEY: &str = "min_password_len";
const MAX_PASSWORD_LEN_KEY: &str = "max_password_len";
const LOGIN_SUCCESSFUL_ADDRESS: &str = "/api/login_successful";

fn get_default_login_data() -> BTreeMap<&'static str, String> {
    let mut data: BTreeMap<&str, String> = BTreeMap::new();
    let min_password_length = password_service::MIN_PASSWORD_LENGTH.to_string();
    let max_password_length = password_service::MAX_PASSWORD_LENGTH.to_string();
    data.insert(MIN_PASSWORD_LEN_KEY, min_password_length);
    data.insert(MAX_PASSWORD_LEN_KEY, max_password_length);
    data
}

fn login_error(data: &mut BTreeMap<&str, String>, error: &str) -> Html<Template> {
    data.insert(ERROR_KEY, error.to_string());
    Html(Template::render(LOGIN_TEMPLATE_NAME, &data))
}

#[derive(FromForm)]
pub struct LoginForm {
    email: String,
    password: String,
}

#[get("/login")]
pub async fn get_login_page() -> Html<Template> {
    let data: BTreeMap<&str, String> = get_default_login_data();
    Html(Template::render(LOGIN_TEMPLATE_NAME, &data))
}

#[post("/login", data = "<user_input>")]
pub async fn post_login(
    db_pool: &State<Pool<DB>>,
    config: &State<Config>,
    user_input: Form<LoginForm>,
) -> Either<Html<Template>, Redirect> {
    let mut data: BTreeMap<&str, String> = get_default_login_data();

    let mut transaction = match new_transaction(db_pool).await {
        Ok(trans) => trans,
        Err(_) => {
            return Either::Left(login_error(&mut data, ERR_INTERNAL));
        }
    };

    let login_details =
        match login_details_repository::get_by_email(&mut transaction, &user_input.email).await {
            Err(err) => {
                error!("Failed communcating with DB: {:?}", err);
                return Either::Left(login_error(&mut data, ERR_INTERNAL));
            }
            Ok(Some(login_details)) => login_details,
            Ok(None) => {
                // No account exists with the given email.
                return Either::Left(login_error(&mut data, ERR_INVALID_EMAIL_OR_PASSWORD));
            }
        };

    if !password_service::verify_password(
        user_input.password.to_owned(),
        login_details.password.to_owned(),
        login_details.password_nonces,
        config,
    ) {
        // Password incorrect
        return Either::Left(login_error(&mut data, ERR_INVALID_EMAIL_OR_PASSWORD));
    }

    // If we reach here, we now accept the user as authorized with the account

    if !login_details.activated {
        // The account has not yet been activated
        return Either::Left(login_error(&mut data, ERR_ACCOUNT_NOT_ACTIVATED));
    }

    // Password correct
    Either::Right(Redirect::to(LOGIN_SUCCESSFUL_ADDRESS))
}
