use rocket::State;
use sqlx::{types::Uuid, Pool};

use crate::{
    db::{account_repository, new_transaction, DB},
    models::account::Account,
    util::accounts_error::AccountsError,
};

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("An internal error occured")]
    Internal,
    #[error("The account doesn't exist")]
    AccountNotFound,
}

impl From<sqlx::Error> for UserError {
    fn from(_: sqlx::Error) -> Self {
        UserError::Internal
    }
}

impl From<AccountsError> for UserError {
    fn from(_: AccountsError) -> Self {
        UserError::Internal
    }
}

pub async fn get_me(account_id: Uuid, db_pool: &State<Pool<DB>>) -> Result<Account, UserError> {
    let mut transaction = new_transaction(db_pool).await?;

    account_repository::get_account(&mut transaction, account_id)
        .await?
        .ok_or(UserError::AccountNotFound)
}
