use crate::db::DB;
use crate::models::login_provider::LOCAL_LOGIN_PROVIDER;
use crate::models::whitelist::Whitelist;
use crate::util::accounts_error::AccountsResult;
use sqlx::Transaction;

pub async fn get_local_account_by_email(
    transaction: &mut Transaction<'_, DB>,
    email: &str,
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

pub async fn get_all_whitelisted_emails(
    transaction: &mut Transaction<'_, DB>,
) -> AccountsResult<Vec<Whitelist>> {
    Ok(sqlx::query_as!(
        Whitelist,
        r#"
SELECT *
FROM whitelist
    "#
    )
    .fetch_all(transaction)
    .await?)
}

pub async fn add_email_to_local_whitelist(
    transaction: &mut Transaction<'_, DB>,
    email: String,
) -> AccountsResult<Whitelist> {
    Ok(sqlx::query_as!(
        Whitelist,
        r#"
INSERT INTO whitelist (email, login_provider)
VALUES                ($1,    'local')
RETURNING *
        "#,
        email
    )
    .fetch_one(transaction)
    .await?)
}
