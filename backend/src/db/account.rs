use crate::db::DB;
use crate::models::account::Account;
use crate::util::accounts_error::AccountsResult;
use sqlx::{PgPool, Transaction};

#[derive(Clone, Debug)]
pub struct AccountRepository {
    pool: PgPool,
}

impl AccountRepository {
    pub fn new(pool: PgPool) -> AccountRepository {
        AccountRepository { pool }
    }

    pub async fn insert(
        &self,
        first_name: &String,
        last_name: &String,
        transaction: &Transaction<'_, DB>,
    ) -> AccountsResult<Account> {
        Ok(sqlx::query_as!(
            Account,
            "
INSERT INTO account (first_name, last_name)
VALUES              ($1,         $2       )
            ",
            first_name,
            last_name,
        )
        .fetch_optional(transaction)
        .await?)
    }
}
