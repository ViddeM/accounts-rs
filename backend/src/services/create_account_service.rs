use crate::db::account_repository;
use crate::db::login_details_repository;
use crate::db::{new_transaction, DB};
use crate::services::password_service;
use crate::util::accounts_error::AccountsError;
use crate::util::config::Config;
use rocket::State;
use sqlx::{Error, Pool};

pub enum CreateAccountError {
    Internal,   // An internal error occurred
    EmailInUse, // The email is already being used
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
    first_name: &String,
    last_name: &String,
    email: &String,
    password: &String,
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

    // TODO: Check whitelist
    let account = match account_repository::insert(&mut transaction, &first_name, &last_name).await
    {
        Ok(acc) => acc,
        Err(err) => {
            error!("Failed to create account {:?}", err);
            transaction.rollback().await?;
            return Err(CreateAccountError::Internal);
        }
    };

    let (password, nonces) = match password_service::hash_and_encrypt_password(&password, &config) {
        Ok(pass) => pass,
        Err(err) => {
            error!("Failed to hash and encrypt password: {:?}", err);
            return Err(CreateAccountError::Internal);
        }
    };

    if let Err(err) =
        login_details_repository::insert(&mut transaction, &account, &email, &password, &nonces)
            .await
    {
        error!("Failed to create login details, err: {:?}", err);
        return Err(CreateAccountError::Internal);
    }

    transaction.commit().await?;
    Ok(())
}
