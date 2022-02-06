use crate::util::accounts_error::{AccountsError, AccountsResult};
use rocket::State;
use sqlx::{Pool, Postgres, Transaction};

pub mod account_repository;
pub mod login_details_repository;
pub mod login_provider_repository;
pub mod third_party_login_repository;
pub mod whitelist_repository;

pub type DB = Postgres;

pub async fn new_transaction(db_pool: &State<Pool<DB>>) -> AccountsResult<Transaction<'_, DB>> {
    match db_pool.begin().await {
        Ok(transaction) => Ok(transaction),
        Err(err) => {
            error!("Failed to create transaction: {:?}", err);
            return Err(AccountsError::SqlxError(err));
        }
    }
}
