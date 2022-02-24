use crate::db::DB;
use crate::services::activate_account_service;
use crate::services::activate_account_service::ActivateAccountError;
use rocket::form::Form;
use rocket::response::content::Html;
use rocket::State;
use rocket_dyn_templates::Template;
use sqlx::Pool;
use std::collections::BTreeMap;
use uuid::Uuid;

const ACTIVATE_ACCOUNT_TEMPLATE_NAME: &str = "activate-account";

const ERROR_KEY: &str = "error";
const INFO_KEY: &str = "info";
const EMAIL_KEY: &str = "email";
const CODE_KEY: &str = "activation_code";

const ERR_INVALID_EMAIL_CODE: &str = "Invalid email or code";
const ERR_INTERNAL: &str = "An internal error has occured, please contact the system administrator";

const INFO_ACTIVATION_SUCCESSFUL: &str =
    "Account activated successfully, you can now login to your account";

#[get("/activate_account")]
pub async fn get_activate_account() -> Html<Template> {
    let data: BTreeMap<&str, &str> = BTreeMap::new();
    Html(Template::render(ACTIVATE_ACCOUNT_TEMPLATE_NAME, data))
}

#[derive(FromForm)]
pub struct ActivateAccountForm {
    email: String,
    code: String,
}

#[post("/activate_account", data = "<activate_account>")]
pub async fn post_activate_account(
    db_pool: &State<Pool<DB>>,
    activate_account: Form<ActivateAccountForm>,
) -> Html<Template> {
    let mut data: BTreeMap<&str, &str> = BTreeMap::new();
    data.insert(EMAIL_KEY, &activate_account.email);
    data.insert(CODE_KEY, &activate_account.code);

    let code = match Uuid::parse_str(&activate_account.code) {
        Err(err) => {
            println!("Failed to parse code to uuid, err: {}", err);
            data.insert(ERROR_KEY, ERR_INVALID_EMAIL_CODE);
            return Html(Template::render(ACTIVATE_ACCOUNT_TEMPLATE_NAME, data));
        }
        Ok(val) => val,
    };

    if let Err(err) =
        activate_account_service::activate_account(&activate_account.email, code, db_pool).await
    {
        match err {
            ActivateAccountError::NoAccountWithEmail | ActivateAccountError::InvalidCode => {
                data.insert(ERROR_KEY, ERR_INVALID_EMAIL_CODE);
            }
            ActivateAccountError::Internal => {
                println!("Internal error occured!");
                data.insert(ERROR_KEY, ERR_INTERNAL);
            }
        }
        return Html(Template::render(ACTIVATE_ACCOUNT_TEMPLATE_NAME, data));
    }

    // TODO: Should probably just redirect to the login screen and let that show this instead.
    data.insert(INFO_KEY, INFO_ACTIVATION_SUCCESSFUL);
    Html(Template::render(ACTIVATE_ACCOUNT_TEMPLATE_NAME, data))
}
