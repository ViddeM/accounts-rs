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
const ERR_INTERNAL: &str = "An internal error occurred";

const LOGIN_SUCCESSFUL_ADDRESS: &str = "/api/login_successful";

#[derive(FromForm)]
pub struct LoginForm {
    email: String,
    password: String,
}

#[get("/login")]
pub async fn get_login_page() -> Html<Template> {
    let data: BTreeMap<&str, String> = BTreeMap::new();
    Html(Template::render(LOGIN_TEMPLATE_NAME, &data))
}

fn login_error(data: &mut BTreeMap<&str, String>, error: &str) -> Html<Template> {
    data.insert(ERROR_KEY, error.to_string());
    Html(Template::render(LOGIN_TEMPLATE_NAME, &data))
}

#[post("/login", data = "<user_input>")]
pub async fn post_login(
    db_pool: &State<Pool<DB>>,
    config: &State<Config>,
    user_input: Form<LoginForm>,
) -> Either<Html<Template>, Redirect> {
    let mut data = BTreeMap::new();

    let mut transaction = match new_transaction(db_pool).await {
        Ok(trans) => trans,
        Err(_) => {
            return Either::Left(login_error(&mut data, ERR_INTERNAL));
        }
    };

    let login_details =
        match login_details_repository::get_by_email(&mut transaction, &user_input.email).await {
            Err(err) => {
                println!("Failed communcating with DB: {:?}", err);
                return Either::Left(login_error(&mut data, ERR_INTERNAL));
            }
            Ok(Some(login_details)) => login_details,
            Ok(None) => {
                error!("No account with email {}", user_input.email);
                return Either::Left(login_error(&mut data, ERR_INVALID_EMAIL_OR_PASSWORD));
            }
        };

    if password_service::verify_password(
        user_input.password.clone(),
        login_details.password.clone(),
        login_details.password_nonces.clone(),
        &config,
    ) == false
    {
        // Password incorrect
        return Either::Left(login_error(&mut data, ERR_INVALID_EMAIL_OR_PASSWORD));
    }

    // Password correct
    return Either::Right(Redirect::to(LOGIN_SUCCESSFUL_ADDRESS));
}
