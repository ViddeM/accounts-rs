use rocket::State;
use sqlx::Pool;
use uuid::Uuid;

use crate::{
    db::{account_repository, login_details_repository, new_transaction, DB},
    models::{account::Account, login_details::LoginDetails},
    util::{accounts_error::AccountsError, uuid::uuid_to_sqlx},
};

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("The account doesn't exist")]
    AccountNotFound,
    #[error("Sqlx error")]
    SqlxError(#[from] sqlx::Error),
    #[error("Accounts error")]
    AccountsError(#[from] AccountsError),
}

pub async fn get_me(
    account_id: Uuid,
    db_pool: &State<Pool<DB>>,
) -> Result<(Account, Option<LoginDetails>), UserError> {
    let mut transaction = new_transaction(db_pool).await?;

    let account_id = uuid_to_sqlx(account_id);

    let acc = account_repository::get_account(&mut transaction, account_id)
        .await?
        .ok_or(UserError::AccountNotFound)?;

    let login_details =
        login_details_repository::get_by_account_id(&mut transaction, account_id).await?;

    transaction.commit().await?;

    Ok((acc, login_details))
}
