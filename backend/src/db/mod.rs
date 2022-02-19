use crate::util::accounts_error::{AccountsError, AccountsResult};
use rocket::State;
use sqlx::{Pool, Postgres, Transaction};

pub mod account_repository;
pub mod login_details_repository;
pub mod login_provider_repository;
pub mod third_party_login_repository;
pub mod whitelist_repository;

use crate::models::login_provider::LOCAL_LOGIN_PROVIDER;

pub type DB = Postgres;
// The query would violate the UNIQUE sql constraint
pub const DB_ERR_VIOLATE_UNIQUE: &str = "23505";

// Initial setup of the database
pub async fn init(pool: &Pool<DB>) -> AccountsResult<()> {
    match sqlx::query_as!(
        LoginProvider,
        "
INSERT INTO login_provider(name)
VALUES                    ($1)
    ",
        LOCAL_LOGIN_PROVIDER
    )
    .execute(pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(sqlx::Error::Database(err)) => {
            if let Some(code) = err.code() {
                if code == DB_ERR_VIOLATE_UNIQUE {
                    // The 'local' login provider already exists
                    return Ok(());
                }
            }
            Err(AccountsError::SqlxError(sqlx::Error::Database(err)))
        }
        Err(err) => Err(AccountsError::SqlxError(err)),
    }
}

pub async fn new_transaction(db_pool: &State<Pool<DB>>) -> AccountsResult<Transaction<'_, DB>> {
    match db_pool.begin().await {
        Ok(transaction) => Ok(transaction),
        Err(err) => {
            error!("Failed to create transaction: {:?}", err);
            Err(AccountsError::SqlxError(err))
        }
    }
}
