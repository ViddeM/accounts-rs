use rocket::State;
use sqlx::Pool;

use crate::{
    db::{new_transaction, whitelist_repository, DB},
    models::whitelist::Whitelist,
    util::accounts_error::AccountsError,
};

#[derive(Debug, thiserror::Error)]
pub enum WhitelistError {
    #[error("An internal error occured")]
    Internal,
}

impl From<sqlx::Error> for WhitelistError {
    fn from(_: sqlx::Error) -> Self {
        WhitelistError::Internal
    }
}

impl From<AccountsError> for WhitelistError {
    fn from(_: AccountsError) -> Self {
        WhitelistError::Internal
    }
}

pub async fn get_whitelisted_emails(
    db_pool: &State<Pool<DB>>,
) -> Result<Vec<Whitelist>, WhitelistError> {
    let mut transaction = new_transaction(db_pool).await?;

    let emails = whitelist_repository::get_all_whitelisted_emails(&mut transaction)
        .await
        .or_else(|err| {
            error!("Failed to get whitelisted emails, err: {}", err);
            Err(err)
        })?;

    Ok(emails)
}

pub async fn add_to_whitelist(
    db_pool: &State<Pool<DB>>,
    email: String,
) -> Result<Whitelist, WhitelistError> {
    let mut transaction = new_transaction(db_pool).await?;

    let email = whitelist_repository::add_email_to_local_whitelist(&mut transaction, email)
        .await
        .or_else(|err| {
            error!("Failed to insert email into local whitelist, err {}", err);
            Err(err)
        })?;

    transaction.commit().await?;

    Ok(email)
}

pub async fn delete_from_whitelist(
    db_pool: &State<Pool<DB>>,
    email: String,
) -> Result<(), WhitelistError> {
    let mut transaction = new_transaction(db_pool).await?;

    whitelist_repository::remove_email(&mut transaction, email)
        .await
        .or_else(|err| {
            error!("Failed to delete email from whitelist, err {}", err);
            Err(err)
        })?;

    transaction.commit().await?;

    Ok(())
}
