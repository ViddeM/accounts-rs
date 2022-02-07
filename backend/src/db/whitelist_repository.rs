use crate::db::DB;
use crate::models::login_provider::LOCAL_LOGIN_PROVIDER;
use crate::models::whitelist::Whitelist;
use crate::util::accounts_error::AccountsResult;
use sqlx::Transaction;

pub async fn get_local_account_by_email(
    transaction: &mut Transaction<'_, DB>,
    email: &String,
) -> AccountsResult<Option<Whitelist>> {
    Ok(sqlx::query_as!(
        Whitelist,
        "
SELECT *
FROM whitelist
WHERE email = $1 AND login_provider = $2
        ",
        email,
        LOCAL_LOGIN_PROVIDER
    )
    .fetch_optional(transaction)
    .await?)
}
