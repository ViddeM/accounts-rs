use crate::db::DB;
use crate::models::login_provider::{LoginProvider, LOCAL_LOGIN_PROVIDER};
use crate::util::accounts_error::AccountsResult;
use sqlx::Transaction;

pub async fn get_local_login_provider(
    transaction: &mut Transaction<'_, DB>,
) -> AccountsResult<LoginProvider> {
    Ok(sqlx::query_as!(
        LoginProvider,
        "
SELECT *
FROM login_provider
WHERE name = $1
        ",
        LOCAL_LOGIN_PROVIDER
    )
    .fetch_one(transaction)
    .await?)
}
