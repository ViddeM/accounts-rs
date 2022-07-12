use crate::db::{account_repository, whitelist_repository};
use crate::db::{activation_code_repository, login_details_repository};
use crate::db::{new_transaction, DB};
use crate::models::activation_code::ActivationCode;
use crate::models::login_details::LoginDetails;
use crate::services::email_service::EmailError;
use crate::services::{email_service, password_service};
use crate::util::accounts_error::AccountsError;
use crate::util::config::Config;
use rocket::State;
use sqlx::{Error, Pool};

#[derive(Debug, thiserror::Error)]
pub enum CreateAccountError {
    #[error("An internal error occured")]
    Internal, // An internal error occurred
    #[error("The email is already in use")]
    EmailInUse, // The email is already being used
    #[error("Email is not in the whitelist")]
    EmailNotWhitelisted, // The email is not in the whitelist
    #[error("Email error")]
    EmailError(#[from] EmailError),
}

impl From<AccountsError> for CreateAccountError {
    fn from(_: AccountsError) -> Self {
        CreateAccountError::Internal
    }
}

impl From<sqlx::Error> for CreateAccountError {
    fn from(_: Error) -> Self {
        CreateAccountError::Internal
    }
}

const ACTIVATE_ACCOUNT_ENDPOINT: &str = "/api/activate_account";

pub async fn create_account(
    config: &State<Config>,
    db_pool: &State<Pool<DB>>,
    first_name: String,
    last_name: String,
    email: String,
    password: String,
) -> Result<(), CreateAccountError> {
    let mut transaction = new_transaction(db_pool).await?;

    let existing_with_email =
        match login_details_repository::get_by_email(&mut transaction, &email).await {
            Ok(val) => val,
            Err(err) => {
                error!("DB err: {:?}", err);
                return Err(CreateAccountError::Internal);
            }
        };

    if existing_with_email.is_some() {
        return Err(CreateAccountError::EmailInUse);
    }

    match whitelist_repository::get_local_account_by_email(&mut transaction, &email).await {
        Ok(Some(_)) => {
            // Is whitelisted so all is fine!
        }
        Ok(None) => {
            // Not whitelisted
            error!(
                "Cannot create account due to email {} not being whitelisted",
                email
            );
            return Err(CreateAccountError::EmailNotWhitelisted);
        }
        Err(err) => {
            error!("DB err: {:?}", err);
            return Err(CreateAccountError::Internal);
        }
    };

    let account = match account_repository::insert(&mut transaction, &first_name, &last_name).await
    {
        Ok(acc) => acc,
        Err(err) => {
            error!("Failed to create account {:?}", err);
            return Err(CreateAccountError::Internal);
        }
    };

    let (hashed_password, nonces) =
        match password_service::hash_and_encrypt_password(password.to_owned(), config) {
            Ok(pass) => pass,
            Err(err) => {
                error!("Failed to hash and encrypt password: {:?}", err);
                return Err(CreateAccountError::Internal);
            }
        };

    let unactivated_account = login_details_repository::create_unactivated_account(
        &mut transaction,
        &account,
        &email,
        &hashed_password,
        &nonces,
    )
    .await
    .or_else(|err| {
        error!("Failed to create login details, err: {:?}", err);
        Err(CreateAccountError::Internal)
    })?;

    let activation_code =
        activation_code_repository::insert(&mut transaction, unactivated_account.account_id)
            .await
            .or_else(|err| {
                error!("Failed to create activation_code, err: {:?}", err);
                Err(CreateAccountError::Internal)
            })?;

    let email_content = format_email_content(config, &unactivated_account, &activation_code);

    // Send email to the email address for confirmation
    email_service::send_email(
        &unactivated_account.email,
        "Aktivera ditt accounts-rs konto",
        // TODO: Make the activation time configurable so that it is correct.
        &email_content,
        config,
    )
    .await?;

    transaction.commit().await?;
    Ok(())
}

fn format_email_content(
    config: &Config,
    unactivated_account: &LoginDetails,
    activation_code: &ActivationCode,
) -> String {
    format!(
        r#"Hi!

An account has been created for accounts-rs with this email but it must be activated before use.
To activate the account, go to the following address: {activate_account_uri}?email={email}&id={code}.

If the account is not activated within 12 hours it will be deleted.
        "#,
        activate_account_uri = format!("{}{}", config.backend_address, ACTIVATE_ACCOUNT_ENDPOINT),
        email = unactivated_account.email,
        code = activation_code.code
    )
}
