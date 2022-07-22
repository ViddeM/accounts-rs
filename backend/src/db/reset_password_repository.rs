use sqlx::types::uuid::Uuid;
use sqlx::Transaction;

use crate::db::DB;
use crate::models::password_reset::PasswordReset;
use crate::util::accounts_error::AccountsResult;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    login_details: Uuid,
) -> AccountsResult<PasswordReset> {
    Ok(sqlx::query_as!(
        PasswordReset,
        "
INSERT INTO password_reset (login_details)
VALUES                     ($1           )
RETURNING id, login_details, code, created_at, modified_at
    ",
        login_details
    )
    .fetch_one(transaction)
    .await?)
}

pub async fn get_by_login_details(
    transaction: &mut Transaction<'_, DB>,
    login_details: Uuid,
) -> AccountsResult<Option<PasswordReset>> {
    Ok(sqlx::query_as!(
        PasswordReset,
        "
SELECT id, login_details, code, created_at, modified_at
FROM password_reset
WHERE login_details = $1
        ",
        login_details
    )
    .fetch_optional(transaction)
    .await?)
}

pub async fn delete_outdated(
    transaction: &mut Transaction<'_, DB>,
    lifetime_minutes: u64,
) -> AccountsResult<Vec<PasswordReset>> {
    Ok(sqlx::query_as!(
        PasswordReset,
        "
DELETE
FROM password_reset
WHERE created_at + ($1 * interval '1 minute') < NOW()
RETURNING id, login_details, code, created_at, modified_at
        ",
        lifetime_minutes as f64
    )
    .fetch_all(transaction)
    .await?)
}

pub async fn get_by_login_details_and_code(
    transaction: &mut Transaction<'_, DB>,
    login_details: Uuid,
    code: Uuid,
) -> AccountsResult<Option<PasswordReset>> {
    Ok(sqlx::query_as!(
        PasswordReset,
        "
SELECT id, login_details, code, created_at, modified_at
FROM password_reset
WHERE
    login_details = $1 AND
    code = $2
    ",
        login_details,
        code
    )
    .fetch_optional(transaction)
    .await?)
}

pub async fn delete_password_reset(
    transaction: &mut Transaction<'_, DB>,
    id: Uuid,
) -> AccountsResult<PasswordReset> {
    Ok(sqlx::query_as!(
        PasswordReset,
        "
DELETE
FROM password_reset
WHERE id=$1
RETURNING id, login_details, code, created_at, modified_at
    ",
        id
    )
    .fetch_one(transaction)
    .await?)
}
