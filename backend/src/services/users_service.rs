use sqlx::Pool;

use crate::{
    db::{account_repository, new_transaction, DB},
    models::account::Account,
    util::accounts_error::AccountsError,
};

#[derive(Debug, thiserror::Error)]
pub enum UsersError {
    #[error("An internal error occured")]
    Internal,
}

impl From<sqlx::Error> for UsersError {
    fn from(_: sqlx::Error) -> Self {
        UsersError::Internal
    }
}

impl From<AccountsError> for UsersError {
    fn from(_: AccountsError) -> Self {
        UsersError::Internal
    }
}

pub async fn get_all_users(db_pool: &Pool<DB>) -> Result<Vec<Account>, UsersError> {
    let mut transaction = new_transaction(db_pool).await?;

    let accs = account_repository::get_all_accounts(&mut transaction)
        .await
        .map_err(|err| {
            error!("Failed to retrieve all users, err: {}", err);
            err
        })?;

    Ok(accs)
}
