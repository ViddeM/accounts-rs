use crate::db::DB;
use crate::models::account::Account;
use crate::models::login_details::LoginDetails;
use crate::util::accounts_error::AccountsResult;
use sqlx::Transaction;

pub async fn get_by_email(
    transaction: &mut Transaction<'_, DB>,
    email: &str,
) -> AccountsResult<Option<LoginDetails>> {
    Ok(sqlx::query_as!(
        LoginDetails,
        "
SELECT *
FROM login_details
WHERE email = $1
    ",
        email
    )
    .fetch_optional(transaction)
    .await?)
}

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    account: &Account,
    email: &str,
    password: &str,
    password_nonces: &str,
) -> AccountsResult<LoginDetails> {
    Ok(sqlx::query_as!(
        LoginDetails,
        "
INSERT INTO login_details (account_id, email, password, password_nonces)
VALUES                    ($1,         $2,    $3,       $4)
RETURNING account_id, email, password, password_nonces, created_at, modified_at
        ",
        account.id,
        email,
        password,
        password_nonces,
    )
    .fetch_one(transaction)
    .await?)
}
