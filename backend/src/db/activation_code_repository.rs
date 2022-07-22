use crate::db::DB;
use crate::models::activation_code::ActivationCode;
use crate::util::accounts_error::AccountsResult;
use sqlx::types::uuid::Uuid;
use sqlx::Transaction;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    login_details: Uuid,
) -> AccountsResult<ActivationCode> {
    Ok(sqlx::query_as!(
        ActivationCode,
        "
INSERT INTO activation_code (login_details)
VALUES                      ($1           )
RETURNING id, login_details, code, created_at, modified_at
        ",
        login_details
    )
    .fetch_one(transaction)
    .await?)
}

pub async fn delete_outdated(
    transaction: &mut Transaction<'_, DB>,
    lifetime_minutes: u64,
) -> AccountsResult<Vec<ActivationCode>> {
    Ok(sqlx::query_as!(
        ActivationCode,
        "
DELETE
FROM activation_code
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
) -> AccountsResult<Option<ActivationCode>> {
    Ok(sqlx::query_as!(
        ActivationCode,
        "
SELECT id, login_details, code, created_at, modified_at
FROM activation_code
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

pub async fn delete(
    transaction: &mut Transaction<'_, DB>,
    login_details: Uuid,
) -> AccountsResult<()> {
    sqlx::query_as!(
        ActivationCode,
        "
DELETE
FROM activation_code
WHERE login_details = $1
    ",
        login_details
    )
    .execute(transaction)
    .await?;
    Ok(())
}
