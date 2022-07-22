use std::collections::BTreeMap;

use rocket::{form::Form, response::Redirect, Either, State};
use rocket_dyn_templates::Template;
use sqlx::types::uuid::Uuid;
use sqlx::Pool;

use crate::{
    db::DB,
    services::reset_password_service::{self, ResetPasswordError},
    util::config::Config,
};

use super::password_validation::{get_default_password_page_data, validate_new_passwords};

const FORGOT_PASSWORD_TEMPLATE_NAME: &str = "forgot-password";
const RESET_PASSWORD_TEMPLATE_NAME: &str = "reset-password";

const INFO_KEY: &str = "info";
const EMAIL_KEY: &str = "email";
const CODE_KEY: &str = "code";
const ERROR_KEY: &str = "error";

const EMAIL_MIGHT_HAVE_BEEN_SENT: &str = "If an account with that email exists, an email has been sent to that address with a code to reset the password of the account.";
const INVALID_EMAIL_OR_CODE: &str =
    "Either the email or the code is invalid or the code is expired";
const PASSWORD_RESET_SUCCESSFUL: &str =
    "Password reset was successful, you can now login with your new password";

const ERR_INTERNAL: &str = "An internal error has occured, please contact the page admin";

#[get("/forgot_password")]
pub async fn get_forgot_password() -> Template {
    let data: BTreeMap<&str, String> = BTreeMap::new();
    Template::render(FORGOT_PASSWORD_TEMPLATE_NAME, &data)
}

#[derive(FromForm)]
pub struct ForgotPasswordForm {
    email: String,
}

#[post("/forgot_password", data = "<forgot_password>")]
pub async fn post_forgot_password(
    config: &State<Config>,
    db_pool: &State<Pool<DB>>,
    forgot_password: Form<ForgotPasswordForm>,
) -> Either<Template, Redirect> {
    if let Err(e) = reset_password_service::initiate_password_reset(
        config,
        db_pool,
        forgot_password.email.clone(),
    )
    .await
    {
        println!(
            "An error occured whilst trying to reset password for email {}, err: {:?}",
            &forgot_password.email, e
        );
        let mut data: BTreeMap<&str, &str> = BTreeMap::new();
        data.insert(EMAIL_KEY, &forgot_password.email);
        data.insert(ERROR_KEY, ERR_INTERNAL);
        return Either::Left(Template::render(FORGOT_PASSWORD_TEMPLATE_NAME, &data));
    }

    Either::Right(Redirect::to(format!(
        "/api/reset_password?s=1&email={}",
        forgot_password.email
    )))
}

#[get("/reset_password?<s>&<email>&<code>")]
pub async fn get_reset_password(
    s: Option<String>,
    email: Option<String>,
    code: Option<String>,
) -> Template {
    let mut data = get_default_password_page_data();

    if let Some(state) = s {
        if state == "1" {
            data.insert(INFO_KEY, EMAIL_MIGHT_HAVE_BEEN_SENT.to_string());
        }
    }

    data.insert(EMAIL_KEY, email.unwrap_or(String::new()));
    data.insert(CODE_KEY, code.unwrap_or(String::new()));

    Template::render(RESET_PASSWORD_TEMPLATE_NAME, &data)
}

#[derive(FromForm)]
pub struct PasswordResetForm {
    email: String,
    code: String,
    new_password: String,
    new_password_repeat: String,
}

#[post("/reset_password", data = "<reset_password>")]
pub async fn post_reset_password(
    config: &State<Config>,
    db_pool: &State<Pool<DB>>,
    reset_password: Form<PasswordResetForm>,
) -> Template {
    let mut data = get_default_password_page_data();

    data.insert(EMAIL_KEY, reset_password.email.clone());
    data.insert(CODE_KEY, reset_password.code.clone());

    let code = match Uuid::parse_str(&reset_password.code) {
        Err(err) => {
            error!("Failed to parse password reset code to uuid, err: {}", err);
            data.insert(ERROR_KEY, INVALID_EMAIL_OR_CODE.to_string());
            return Template::render(RESET_PASSWORD_TEMPLATE_NAME, &data);
        }
        Ok(val) => val,
    };

    if let Err(e) = validate_new_passwords(
        &reset_password.new_password,
        &reset_password.new_password_repeat,
    ) {
        data.insert(ERROR_KEY, e.to_string());
        return Template::render(RESET_PASSWORD_TEMPLATE_NAME, &data);
    }

    if let Err(e) = reset_password_service::update_password(
        config,
        db_pool,
        reset_password.email.to_owned(),
        code,
        reset_password.new_password.to_owned(),
    )
    .await
    {
        data.insert(ERROR_KEY, e.to_api_err().to_string());
        return Template::render(RESET_PASSWORD_TEMPLATE_NAME, &data);
    }

    data.insert(INFO_KEY, PASSWORD_RESET_SUCCESSFUL.to_string());
    Template::render(RESET_PASSWORD_TEMPLATE_NAME, &data)
}

impl ResetPasswordError {
    fn to_api_err<'a>(&self) -> &'a str {
        match self {
            ResetPasswordError::Internal => ERR_INTERNAL,
            ResetPasswordError::EmailError(_) => ERR_INTERNAL,
            ResetPasswordError::InvalidEmailOrCode => INVALID_EMAIL_OR_CODE,
        }
    }
}
