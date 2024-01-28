use crate::db::DB;
use crate::models::login_provider::{LoginProvider, LOCAL_LOGIN_PROVIDER};
use crate::util::accounts_error::AccountsResult;
use sqlx::Transaction;

// TODO: We'll probably need this when we have multiple login providers
#[allow(dead_code)]
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
    .fetch_one(&mut **transaction)
    .await?)
}
