use crate::db::{account_repository, whitelist_repository};
use crate::db::{activation_code_repository, login_details_repository};
use crate::db::{new_transaction, DB};
use crate::services::password_service;
use crate::util::accounts_error::AccountsError;
use crate::util::config::Config;
use rocket::State;
use sqlx::{Error, Pool};

pub enum CreateAccountError {
    Internal,            // An internal error occurred
    EmailInUse,          // The email is already being used
    EmailNotWhitelisted, // The email is not in the whitelist
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

    let unactived_account = login_details_repository::create_unactivated_account(
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

    if let Err(err) =
        activation_code_repository::insert(&mut transaction, unactived_account.account_id).await
    {
        error!("Failed to create activation_code, err: {:?}", err);
        return Err(CreateAccountError::Internal);
    }

    // Send email to the email address for confirmation

    transaction.commit().await?;
    Ok(())
}