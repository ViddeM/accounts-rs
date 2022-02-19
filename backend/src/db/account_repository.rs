use crate::db::DB;
use crate::models::account::Account;
use crate::util::accounts_error::AccountsResult;
use sqlx::Transaction;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    first_name: &str,
    last_name: &str,
) -> AccountsResult<Account> {
    Ok(sqlx::query_as!(
        Account,
        "
INSERT INTO account (first_name, last_name)
VALUES              ($1,         $2       )
RETURNING id, first_name, last_name, created_at, modified_at
        ",
        first_name,
        last_name,
    )
    .fetch_one(transaction)
    .await?)
}
