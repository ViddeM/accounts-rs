use crate::db::DB;
use crate::services::activate_account_service;
use crate::services::activate_account_service::ActivateAccountError;
use rocket::State;
use rocket_dyn_templates::Template;
use sqlx::types::uuid::Uuid;
use sqlx::Pool;
use std::collections::BTreeMap;

const ACTIVATE_ACCOUNT_TEMPLATE_NAME: &str = "activate-account";

const ERROR_KEY: &str = "error";
const INFO_KEY: &str = "info";

const ERR_INVALID_EMAIL_CODE: &str = "Invalid activation link";
const ERR_INTERNAL: &str = "An internal error has occured, please contact the system administrator";

const INFO_ACTIVATION_SUCCESSFUL: &str =
    "Account activated successfully, you can now login to your account";

#[get("/activate_account?<email>&<id>")]
pub async fn get_activate_account(
    db_pool: &State<Pool<DB>>,
    email: Option<String>,
    id: Option<String>,
) -> Template {
    let mut data: BTreeMap<&str, &str> = BTreeMap::new();

    if email.is_none() || id.is_none() {
        error!("Email or code was empty");
        data.insert(ERROR_KEY, ERR_INVALID_EMAIL_CODE);
        return Template::render(ACTIVATE_ACCOUNT_TEMPLATE_NAME, data);
    }

    let email = email.unwrap_or_default();
    let id = id.unwrap_or_default();

    let code = match Uuid::parse_str(&id) {
        Err(err) => {
            error!("Failed to parse code to uuid, err: {}", err);
            data.insert(ERROR_KEY, ERR_INVALID_EMAIL_CODE);
            return Template::render(ACTIVATE_ACCOUNT_TEMPLATE_NAME, data);
        }
        Ok(val) => val,
    };

    if let Err(err) = activate_account_service::activate_account(&email, code, db_pool).await {
        match err {
            ActivateAccountError::NoAccountWithEmail | ActivateAccountError::InvalidCode => {
                data.insert(ERROR_KEY, ERR_INVALID_EMAIL_CODE);
            }
            ActivateAccountError::Internal => {
                error!("Internal error occured!");
                data.insert(ERROR_KEY, ERR_INTERNAL);
            }
        }
        return Template::render(ACTIVATE_ACCOUNT_TEMPLATE_NAME, data);
    }

    // TODO: Should probably just redirect to the login screen and let that show this instead.
    data.insert(INFO_KEY, INFO_ACTIVATION_SUCCESSFUL);
    Template::render(ACTIVATE_ACCOUNT_TEMPLATE_NAME, data)
}
