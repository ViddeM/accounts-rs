use crate::db::DB;
use crate::models::account::Account;
use crate::models::login_details::LoginDetails;
use crate::util::accounts_error::AccountsResult;
use sqlx::Transaction;
use uuid::Uuid;

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

pub async fn create_unactivated_account(
    transaction: &mut Transaction<'_, DB>,
    account: &Account,
    email: &str,
    password: &str,
    password_nonces: &str,
) -> AccountsResult<LoginDetails> {
    Ok(sqlx::query_as!(
        LoginDetails,
        "
INSERT INTO login_details (account_id, email, password, password_nonces, activated)
VALUES                    ($1,         $2,    $3,       $4,              false)
RETURNING account_id, email, password, password_nonces, created_at, modified_at, activated
        ",
        account.id,
        email,
        password,
        password_nonces,
    )
    .fetch_one(transaction)
    .await?)
}

pub async fn delete_multiple(
    transaction: &mut Transaction<'_, DB>,
    account_ids: &[Uuid],
) -> AccountsResult<()> {
    sqlx::query_as!(
        LoginDetails,
        "
DELETE 
FROM login_details
WHERE account_id = ANY($1)
        ",
        account_ids
    )
    .execute(transaction)
    .await?;
    Ok(())
}
