use crate::api::core::activate_account::rocket_uri_macro_get_activate_account;
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
use sqlx::Pool;

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
    fn from(_: sqlx::Error) -> Self {
        CreateAccountError::Internal
    }
}

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
    .map_err(|err| {
        error!("Failed to create login details, err: {:?}", err);
        CreateAccountError::Internal
    })?;

    let activation_code =
        activation_code_repository::insert(&mut transaction, unactivated_account.account_id)
            .await
            .map_err(|err| {
                error!("Failed to create activation_code, err: {:?}", err);
                CreateAccountError::Internal
            })?;

    let email_content = format_email_content(config, &unactivated_account, &activation_code);

    // Send email to the email address for confirmation
    if let Err(e) = email_service::send_email(
        &unactivated_account.email,
        "Activate your accounts-rs account",
        // TODO: Make the activation time configurable so that it is correct.
        &email_content,
        config,
    )
    .await
    {
        error!("Failed to send email, err: {}", e);
        return Err(CreateAccountError::Internal);
    }

    transaction.commit().await?;
    Ok(())
}

fn format_email_content(
    config: &Config,
    unactivated_account: &LoginDetails,
    activation_code: &ActivationCode,
) -> String {
    let activate_account_uri = format!(
        "{}/api/core{}",
        config.backend_address,
        uri!(get_activate_account(
            Some(unactivated_account.email.clone()),
            Some(activation_code.code.to_string())
        ))
        .to_string(),
    );

    format!(
        r#"Hi!

An account has been created for accounts-rs with this email but it must be activated before use.
To activate the account, go to the following address: {activate_account_uri}.

If the account is not activated within 12 hours it will be deleted.

If you did not register to accounts-rs, please ignore this email!
        "#,
        activate_account_uri = activate_account_uri,
    )
}
