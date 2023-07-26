use crate::api::auth::session_guard::{set_session, Session};
use crate::db::DB;
use crate::services::login_service::LoginError;
use crate::services::{login_service, password_service};
use crate::util::config::Config;
use mobc_redis::RedisConnectionManager;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::{Either, State};
use rocket_dyn_templates::Template;
use std::collections::BTreeMap;

const LOGIN_TEMPLATE_NAME: &str = "login";

const ERROR_KEY: &str = "error";

const ERR_INVALID_EMAIL_OR_PASSWORD: &str =
    "Invalid email or password or the account has been locked due to excessive incorrect passwords";
const ERR_ACCOUNT_NOT_ACTIVATED: &str = "The account has not yet been activated";
const ERR_INTERNAL: &str = "An internal error occurred";

const MIN_PASSWORD_LEN_KEY: &str = "min_password_len";
const MAX_PASSWORD_LEN_KEY: &str = "max_password_len";
const LOGIN_SUCCESSFUL_ADDRESS: &str = "/";
const EMAIL_KEY: &str = "email";

const RETURN_TO_KEY: &str = "return_to";

fn get_default_login_data() -> BTreeMap<&'static str, String> {
    let mut data: BTreeMap<&str, String> = BTreeMap::new();
    let min_password_length = password_service::MIN_PASSWORD_LENGTH.to_string();
    let max_password_length = password_service::MAX_PASSWORD_LENGTH.to_string();
    data.insert(MIN_PASSWORD_LEN_KEY, min_password_length);
    data.insert(MAX_PASSWORD_LEN_KEY, max_password_length);
    data
}

fn login_error(data: &mut BTreeMap<&str, String>, error: &str) -> Template {
    data.insert(ERROR_KEY, error.to_string());
    Template::render(LOGIN_TEMPLATE_NAME, &data)
}

#[get("/login?<return_to>")]
pub async fn get_login_page(
    return_to: Option<String>,
    session: Option<Session>,
) -> Either<Template, Redirect> {
    if session.is_some() {
        return Either::Right(Redirect::to(LOGIN_SUCCESSFUL_ADDRESS));
    }

    let mut data: BTreeMap<&str, String> = get_default_login_data();
    if let Some(return_to) = return_to {
        data.insert(RETURN_TO_KEY, return_to);
    }

    Either::Left(Template::render(LOGIN_TEMPLATE_NAME, data))
}

#[derive(FromForm)]
pub struct LoginForm {
    email: String,
    password: String,
    return_to: Option<String>,
}

#[post("/login", data = "<user_input>")]
pub async fn post_login(
    db_pool: &State<sqlx::Pool<DB>>,
    config: &State<Config>,
    user_input: Form<LoginForm>,
    cookies: &CookieJar<'_>,
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
) -> Either<Template, Redirect> {
    let mut data: BTreeMap<&str, String> = get_default_login_data();

    let login_details = match login_service::validate_login(
        config,
        db_pool,
        user_input.email.clone(),
        user_input.password.clone(),
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            let err = match e {
                LoginError::Internal => ERR_INTERNAL,
                LoginError::InvalidEmailPassword => ERR_INVALID_EMAIL_OR_PASSWORD,
                LoginError::AccountLocked => ERR_INVALID_EMAIL_OR_PASSWORD,
                LoginError::AccountNotActivated => ERR_ACCOUNT_NOT_ACTIVATED,
            };

            data.insert(EMAIL_KEY, user_input.email.clone());
            return Either::Left(login_error(&mut data, err));
        }
    };

    if let Err(e) = set_session(redis_pool, &login_details, cookies).await {
        error!("Failed to set session for login {}", e);
        return Either::Left(login_error(&mut data, ERR_INTERNAL));
    }

    let redirect_address = match user_input.return_to.clone() {
        Some(a) => a,
        None => String::from(LOGIN_SUCCESSFUL_ADDRESS),
    };

    Either::Right(Redirect::to(redirect_address))
}
